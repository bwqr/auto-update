use std::collections::HashMap;

use actix::{Actor, Addr, Context, Handler, Message};
use log::error;
use shared::Command;

use crate::session::{Session, SessionId, SessionMessage};

pub struct ConnectServer {
    pub session_id: SessionId,
    pub addr: Addr<Session>
}
impl Message for ConnectServer {
    type Result = ();
}

pub struct DisconnectServer(pub SessionId);
impl Message for DisconnectServer {
    type Result = ();
}

pub struct SendCommand {
    pub session_id: SessionId,
    pub command: Command
}
impl Message for SendCommand {
    type Result = ();
}

pub struct MessageServer {
    clients: HashMap<SessionId, Addr<Session>>
}

impl MessageServer {
    pub fn new() -> Self {
        MessageServer {
            clients: HashMap::new()
        }
    }
}

impl Actor for MessageServer {
    type Context = Context<Self>;
}

impl Handler<ConnectServer> for MessageServer {
    type Result = <ConnectServer as Message>::Result;

    fn handle(&mut self, msg: ConnectServer, _: &mut Self::Context) -> Self::Result {
        self.clients.insert(msg.session_id, msg.addr);
    }
}

impl Handler<DisconnectServer> for MessageServer {
    type Result = <DisconnectServer as Message>::Result;

    fn handle(&mut self, msg: DisconnectServer, _: &mut Self::Context) -> Self::Result {
        self.clients.remove(&msg.0);
    }
}

impl Handler<SendCommand> for MessageServer {
    type Result = <SendCommand as Message>::Result;

    fn handle(&mut self, msg: SendCommand, _: &mut Self::Context) -> Self::Result {
        if let Some(addr) = self.clients.get(&msg.session_id) {
            addr.do_send(SessionMessage(serde_json::to_string(&msg.command).unwrap()));
        } else {
            error!("trying to send to unknown session, {}", msg.session_id);
        }
    }
}
