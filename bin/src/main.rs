#[macro_use]
extern crate log;

use dotenv::dotenv;
use lib::discord::start;
use lib::print_2b;
use std::fs;
use std::io::ErrorKind;

#[tokio::main]
async fn main() {
    if let Err(e) = create_log_folder() {
        match e.kind() {
            ErrorKind::AlreadyExists => (),
            e => panic!("{:?}", e),
        }
    }
    log4rs::init_file("./config/log4rs.toml", Default::default()).unwrap();
    print_2b();
    dotenv().ok();
    // match start().await {
    //     Ok(_) => (),
    //     Err(e) => error!("{:#?}", e),
    // }
    start().await;
}

fn create_log_folder() -> std::io::Result<()> {
    fs::create_dir("./logs")
}
