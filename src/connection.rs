use std::cmp::min;
use std::time::Duration;

use actix::io::{SinkWrite, WriteHandler};
use actix::{
    Actor, ActorFutureExt, Arbiter, AsyncContext, Context, ContextFutureSpawner,
    StreamHandler, WrapFuture,
};
use actix_codec::Framed;
use awc::error::{WsClientError, WsProtocolError};
use awc::ws::{Codec, Frame, Message};
use awc::{BoxedSocket, Client};
use futures::stream::SplitSink;
use futures::StreamExt;
use log::{error, info};
use shared::Command;

type Write = SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>;

const MAX_TIMING: usize = 5;

const TIMINGS: [u8; MAX_TIMING] = [0, 2, 4, 6, 8];

#[derive(Debug)]
enum Error {
    InvalidMessage,
}

pub struct Connection {
    sink: Option<Write>,
    url: String,
    timing_index: usize,
    worker_arbiter: Arbiter,
}

impl Connection {
    pub fn new(url: String, arbiter: Arbiter) -> Self {
        Connection {
            sink: None,
            url,
            timing_index: 0,
            worker_arbiter: arbiter,
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
                        act.timing_index = min(act.timing_index + 1, MAX_TIMING - 1);

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

    fn handle_frame(&mut self, frame: Frame) -> Result<(), Error> {
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
                    Command::Restart => {
                        info!("stopping worker");
                        self.worker_arbiter.stop();
                    },
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
}

impl StreamHandler<Result<Frame, WsProtocolError>> for Connection {
    fn handle(&mut self, item: Result<Frame, WsProtocolError>, _: &mut Self::Context) {
        if let Ok(frame) = item {
            if let Err(e) = self.handle_frame(frame) {
                error!("failed to handle frame, {:?}", e);
            }
        }
    }

    fn finished(&mut self, ctx: &mut Context<Self>) {
        error!("connection is closed");
        self.sink = None;
        self.try_connect(ctx);
    }
}

impl WriteHandler<WsProtocolError> for Connection {}
