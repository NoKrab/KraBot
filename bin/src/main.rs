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
    debug!("cargo bin version: {}", env!("CARGO_PKG_VERSION"));
    debug!("build Timestamp: {}", env!("VERGEN_BUILD_TIMESTAMP"));
    debug!(
        "git branch: {}-{}",
        env!("VERGEN_GIT_BRANCH"),
        env!("VERGEN_GIT_SHA")
    );
    print_2b();
    dotenv().ok();
    if let Err(e) = start().await {
        error!("{}", e)
    }
}

fn create_log_folder() -> std::io::Result<()> {
    fs::create_dir("./logs")
}
