use std::fmt::Display;

use actix::{Actor, Addr};
use actix_web::{
    post,
    get, middleware,
    web::{self, Json},
    App, HttpServer, ResponseError, Result as ActixResult,
};
use message_server::MessageServer;

use crate::{message_server::SendMessage, session::{Session, SessionId}, session_manager::{CreateSessionId, ListSessions, SessionManager}};

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

#[post("/message/{id}/{message}")]
async fn send_message(req: web::HttpRequest, paths: web::Path<(SessionId, String)>) -> Result<Json<()>, Error> {
    let message_server = req.app_data::<Addr<MessageServer>>().unwrap();
    message_server
        .send(SendMessage {session_id: paths.0, message: paths.1.clone()})
        .await
        .map_err(|_| Error::Mailbox)?;

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
