use std::cmp::min;
use std::time::Duration;
use std::os::unix::fs::PermissionsExt;

use actix::io::{SinkWrite, WriteHandler};
use actix::{Actor, ActorFutureExt, AsyncContext, Context, ContextFutureSpawner, StreamHandler, System, WrapFuture};
use actix_codec::Framed;
use async_std::io::WriteExt;
use awc::error::{WsClientError, WsProtocolError};
use awc::ws::{Codec, Frame, Message};
use awc::{BoxedSocket, Client};
use futures::stream::SplitSink;
use futures::StreamExt;
use log::{error, info};
use shared::Command;

type Write = SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>;

const MAX_TIMING: usize = 4;

const TIMINGS: [u8; MAX_TIMING + 1] = [0, 2, 4, 6, 8];

#[derive(Debug)]
enum Error {
    InvalidMessage,
}

pub struct Connection {
    sink: Option<Write>,
    url: String,
    timing_index: usize,
    restarting: bool,
}

impl Connection {
    pub fn new(url: String) -> Self {
        Connection {
            sink: None,
            url,
            timing_index: 0,
            restarting: false,
        }
    }

    async fn connect(url: String) -> Result<Framed<BoxedSocket, Codec>, WsClientError> {
        Client::new()
            .ws(url)
            .connect()
            .await
            .map(|(_, framed)| framed)
    }

    fn try_connect(&mut self, ctx: &mut <Self as Actor>::Context) {
        Self::connect(self.url.clone())
            .into_actor(self)
            .then(|framed, act, ctx| {
                match framed {
                    Ok(framed) => {
                        info!("connected to server");

                        let (sink, stream) = framed.split();
                        Self::add_stream(stream, ctx);
                        act.sink = Some(SinkWrite::new(sink, ctx));
                        act.timing_index = 0;
                    }
                    Err(e) => {
                        act.timing_index = min(act.timing_index + 1, MAX_TIMING);

                        error!("{:?}", e);
                        error!(
                            "failed to connect to server, will retry in {}",
                            TIMINGS[act.timing_index]
                        );

                        ctx.run_later(
                            Duration::from_secs(TIMINGS[act.timing_index] as u64),
                            |act, ctx| Self::try_connect(act, ctx),
                        );
                    }
                }

                actix::fut::ready(())
            })
            .spawn(ctx);
    }

    async fn download_file(url: &'static str, file_path: &'static str) -> Result<(), String> {
        let mut resp = awc::Client::new()
            .get(url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("failed to download, {:?}", e))?;

        if resp.status() != 200 {
            return Err(format!(
                "server returned unsuccessful message, {:?}",
                resp.body().await
            ));
        }

        let mut file = async_std::fs::File::create(file_path)
            .await
            .map_err(|e| format!("failed to create file, {:?}", e))?;

        let mut perms = file.metadata()
            .await
            .map_err(|e| format!("failed to read metadata of file, {:?}", e))?
            .permissions();
        perms.set_mode(0o755);

        file.set_permissions(perms)
            .await
            .map_err(|e| format!("failed to set permission of file, {:?}", e))?;

        while let Some(chunk) = resp.next().await {
            let bytes = chunk.map_err(|e| format!("malformed chunk, {:?}", e))?;

            file.write_all(&bytes)
                .await
                .map_err(|e| format!("failed to write chunks into file, {:?}", e))?;
        }

        Ok(())
    }

    fn restart(&mut self, ctx: &mut <Self as Actor>::Context) {
        self.restarting = true;

        async {
            if let Err(e) = Self::download_file("http://127.0.0.1:8080/updater", "storage/client/updater").await {
                error!("failed to download updater, {:?}", e);
                return false;
            }
            info!("downloaded the updater");

            if let Err(e) = Self::download_file("http://127.0.0.1:8080/app", "storage/client/app").await {
                error!("failed to download app, {:?}", e);
                return false;
            }
            info!("downloaded the app");

            return true;
        }
            .into_actor(self)
            .then(|res, act, _| {
                if res {
                    info!("downloads are successful, restarting to update");

                    let proc = std::process::Command::new("storage/client/updater")
                        .spawn();

                    match proc {
                        Ok(_) => info!("spawned the updater"),
                        Err(e) => error!("failed to spawn updater, {:?}", e)
                    };

                    System::current().stop();
                    info!("stopped the system");
                } else {
                    act.restarting = false;
                }

                async {}.into_actor(act)
            })
            .spawn(ctx);
    }

    fn handle_frame(
        &mut self,
        frame: Frame,
        ctx: &mut <Self as Actor>::Context,
    ) -> Result<(), Error> {
        match frame {
            Frame::Ping(msg) => {
                if let Some(sink) = &mut self.sink {
                    let _ = sink.write(Message::Pong(msg));
                }
            }
            Frame::Pong(_) => {}
            Frame::Text(msg) => {
                info!("recevied the msg {:?}", msg);
                let command =
                    serde_json::from_slice::<Command>(&msg).map_err(|_| Error::InvalidMessage)?;

                match command {
                    Command::Update => {
                        if self.restarting {
                            info!("received restart message while restarting");
                            return Ok(());
                        }
                        self.restart(ctx);
                    }
                    Command::Dummy => info!("received dummy command"),
                }
            }
            _ => {}
        };

        Ok(())
    }
}

impl Actor for Connection {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("connection is started");
        info!("thread id {:?}", std::thread::current().id());
        self.try_connect(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> actix::Running {
        info!("stopping connection");
        actix::Running::Stop
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        info!("connection is stopped");
    }
}

impl StreamHandler<Result<Frame, WsProtocolError>> for Connection {
    fn handle(&mut self, item: Result<Frame, WsProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(frame) => {
                if let Err(e) = self.handle_frame(frame, ctx) {
                    error!("failed to handle frame, {:?}", e);
                }
            }
            Err(e) => error!("ws protocol error occured, {:?}", e),
        }
    }

    fn finished(&mut self, ctx: &mut Context<Self>) {
        error!("connection is closed");
        self.sink = None;
        self.try_connect(ctx);
    }
}

impl WriteHandler<WsProtocolError> for Connection {}
