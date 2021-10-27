use actix::{Actor, Arbiter, System};
use connection::Connection;
use worker::Worker;

mod connection;
mod worker;

fn main() {
    std::env::set_var("RUST_LOG", "info");

    env_logger::init();

    log::info!("main thread id {:?}", std::thread::current().id());

    let sys = System::new();

    let worker_arbiter = Arbiter::new();
    let worker = Worker::start_in_arbiter(&worker_arbiter.handle(), |_| Worker::new());

    Connection::start_in_arbiter(&Arbiter::new().handle(), |_| Connection::new(String::from("http://127.0.0.1:8080/ws"), worker_arbiter));

    sys.run().unwrap();
}
