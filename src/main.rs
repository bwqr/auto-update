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

    Worker::start_in_arbiter(&Arbiter::new().handle(), |_| Worker::new());

    Connection::start_in_arbiter(&Arbiter::new().handle(), |_| Connection::new(String::from("http://127.0.0.1:8080/ws")));

    sys.run().unwrap();
}
