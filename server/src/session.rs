use actix::{Actor, Addr, AsyncContext, Handler, Message, StreamHandler};
use actix_web_actors::ws::{Message as WebsocketMessage, ProtocolError, WebsocketContext};

use crate::{message_server::{ConnectServer, DisconnectServer, MessageServer}, session_manager::{RemoveSessionId, SessionManager}};

pub type SessionId = u64;

pub struct SessionMessage(pub String);
impl Message for SessionMessage {
    type Result = ();
}

pub struct Session {
    id: SessionId,
    manager: Addr<SessionManager>,
    server: Addr<MessageServer>,
}

impl Session {
    pub fn new (id: SessionId, manager: Addr<SessionManager>, server: Addr<MessageServer>) -> Self {
        Session {
            id,
            manager,
            server
        }
    }
}

impl Actor for Session {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.server.do_send(ConnectServer { session_id: self.id, addr: ctx.address() });
    }
}

impl StreamHandler<Result<WebsocketMessage, ProtocolError>> for Session {
    fn handle(&mut self, item: Result<WebsocketMessage, ProtocolError>, ctx: &mut Self::Context) {
        todo!()
    }

    fn finished(&mut self, _: &mut Self::Context) {
        self.manager.do_send(RemoveSessionId(self.id));
        self.server.do_send(DisconnectServer(self.id));
    }
}

impl Handler<SessionMessage> for Session {
    type Result = <SessionMessage as Message>::Result;

    fn handle(&mut self, msg: SessionMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0)
    }
}
