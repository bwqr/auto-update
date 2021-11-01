use log::info;

fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    info!("updater version 4");

    info!("updating the app");
    std::fs::copy("storage/client/app", "storage/client/auto-update").unwrap();
    info!("updated the app successfully");

    info!("starting the app");
    std::process::Command::new("storage/client/auto-update")
        .spawn()
        .unwrap();
    info!("started the app successfully");
}
