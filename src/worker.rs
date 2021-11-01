use std::time::Duration;

use actix::{Actor, AsyncContext, Context};
use log::info;

pub struct Worker {
    iteration: u64,
}

impl Worker {
    pub fn new() -> Self {
        Worker {
            iteration: 0,
        }
    }

    fn do_work(&mut self, ctx: &mut <Self as Actor>::Context) {
        const INTERVAL_TIME: u64 = 10;

        self.iteration += 1;

        info!("doing some work");

        std::thread::sleep(Duration::from_secs(10));

        info!("work done, will run in {} seconds, iteration {}", INTERVAL_TIME, self.iteration);

        ctx.run_later(Duration::from_secs(INTERVAL_TIME), Self::do_work);
    }
}

impl Actor for Worker {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("worker is started");
        info!("thread id {:?}", std::thread::current().id());

        self.do_work(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> actix::Running {
        info!("stopping worker");
        actix::Running::Stop
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        info!("worker is stopped");
    }
}
