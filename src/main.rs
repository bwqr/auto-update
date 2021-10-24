use actix::{Actor, Arbiter, System};
use connection::Connection;

mod connection;

fn main() {
    std::env::set_var("RUST_LOG", "info");

    env_logger::init();

    let sys = System::new();

    Arbiter::new().spawn(async move {
        Connection::new(String::from("http://127.0.0.1:8080/ws")).start();
    });

    sys.run().unwrap();
}
