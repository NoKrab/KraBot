use clap::{App, ArgMatches};

pub fn setup_cli() -> ArgMatches {
    App::new("KraBot")
        .about("Discord Bot written in Rust")
        .version(&*version())
        .get_matches()
}

fn version() -> String {
    format!(
        "{}\n{}-{}\n{}",
        env!("CARGO_PKG_VERSION"),
        env!("VERGEN_GIT_BRANCH"),
        env!("VERGEN_GIT_SHA"),
        env!("VERGEN_BUILD_TIMESTAMP")
    )
}
