use clap::{crate_version, App, ArgMatches};

pub fn setup_cli() -> ArgMatches {
    App::new("KraBot")
        .about("Discord Bot written in Rust")
        .version(&*version())
        .get_matches()
}

fn version() -> String {
    format!(
        "{}\n{}-{}\n{}",
        crate_version!(),
        env!("VERGEN_GIT_BRANCH"),
        env!("VERGEN_GIT_SHA"),
        env!("VERGEN_BUILD_TIMESTAMP")
    )
}
