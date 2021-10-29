use std::fmt::Display;

use actix::{Actor, Addr};
use actix_web::{
    get, middleware, post,
    web::{self, Json},
    App, HttpServer, ResponseError, Result as ActixResult,
};
use serde::Deserialize;

use message_server::MessageServer;
use shared::Command;

use crate::{
    message_server::SendCommand,
    session::Session,
    session_manager::{CreateSessionId, ListSessions, SessionManager},
};

mod message_server;
mod session;
mod session_manager;

#[derive(Debug)]
enum Error {
    Mailbox,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Mailbox => f.write_str("Mailbox"),
        }
    }
}

impl ResponseError for Error {}

#[get("/ws")]
async fn connect(req: web::HttpRequest, stream: web::Payload) -> ActixResult<web::HttpResponse> {
    let session_manager = req.app_data::<Addr<SessionManager>>().unwrap();
    let message_server = req.app_data::<Addr<MessageServer>>().unwrap();

    let session_id = session_manager
        .send(CreateSessionId)
        .await
        .map_err(|_| Error::Mailbox)?;

    let session = Session::new(session_id, session_manager.clone(), message_server.clone());

    actix_web_actors::ws::start(session, &req, stream)
}

#[get("/sessions")]
async fn sessions(req: web::HttpRequest) -> Result<Json<Vec<u64>>, Error> {
    req.app_data::<Addr<SessionManager>>()
        .unwrap()
        .send(ListSessions)
        .await
        .map_err(|_| Error::Mailbox)
        .map(|sessions| Json(sessions))
}

#[derive(Deserialize)]
struct CommandQuery {
    command: Command,
}

#[post("/message")]
async fn send_message(
    req: web::HttpRequest,
    command: web::Query<CommandQuery>,
) -> Result<Json<()>, Error> {
    let command = command.into_inner().command;

    let message_server = req.app_data::<Addr<MessageServer>>().unwrap();

    req.app_data::<Addr<SessionManager>>()
            .unwrap()
            .send(ListSessions)
            .await
            .map_err(|_| Error::Mailbox)?
            .into_iter()
            .for_each(|session| message_server.do_send(SendCommand {session_id: session, command: command.clone()}));

    Ok(Json(()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");

    env_logger::init();

    let session_manager = SessionManager::new().start();
    let message_server = MessageServer::new().start();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(session_manager.clone())
            .app_data(message_server.clone())
            .service(connect)
            .service(sessions)
            .service(send_message)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
