use std::collections::HashSet;

use actix::{Actor, Context, Handler, Message};

use rand::{Rng, prelude::ThreadRng};

use crate::session::SessionId;

pub struct CreateSessionId;
impl Message for CreateSessionId {
    type Result = SessionId;
}

pub struct RemoveSessionId(pub SessionId);
impl Message for RemoveSessionId {
    type Result = bool;
}

pub struct ListSessions;
impl Message for ListSessions {
    type Result = Vec<SessionId>;
}

pub struct SessionManager {
    sessions: HashSet<SessionId>,
    rng: ThreadRng,
}

impl SessionManager {
    pub fn new() -> Self {
        SessionManager {
            sessions: HashSet::new(),
            rng: ThreadRng::default()
        }
    }
}

impl Actor for SessionManager {
    type Context = Context<Self>;
}

impl Handler<CreateSessionId> for SessionManager {
    type Result = <CreateSessionId as Message>::Result;

    fn handle(&mut self, _: CreateSessionId, _: &mut Self::Context) -> Self::Result {
        let mut session_id: SessionId = self.rng.gen();

        loop {
            if self.sessions.contains(&session_id) {
                session_id = self.rng.gen();
            } else {
                self.sessions.insert(session_id);

                return session_id;
            }
        }
    }
}

impl Handler<RemoveSessionId> for SessionManager {
    type Result = <RemoveSessionId as Message>::Result;

    fn handle(&mut self, msg: RemoveSessionId, _: &mut Self::Context) -> Self::Result {
        self.sessions.remove(&msg.0)
    }
}

impl Handler<ListSessions> for SessionManager {
    type Result = <ListSessions as Message>::Result;

    fn handle(&mut self, _: ListSessions, _: &mut Self::Context) -> Self::Result {
        (&self.sessions).into_iter().cloned().collect()
    }
}


