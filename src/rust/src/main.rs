use std::process::Stdio;
use valve::start;

use {argh::FromArgs, std::fmt::Debug};
#[derive(FromArgs, Debug)]
/// Distribute your plumber API in parallel.
struct Cli {
    /// host to serve APIs on
    #[argh(option, short = 'h', default = r#"String::from("127.0.0.1")"#)]
    host: String,

    /// the port to serve the main application on
    #[argh(option, short = 'p', default = "3000")]
    port: u16,

    /// the maximum number of plumber APIs to spawn
    #[argh(option, short = 'n', default = "3")]
    n_max: u16,

    /// number of Tokio workers to spawn to handle requests
    #[argh(option, short = 'w', default = "3")]
    workers: u16,

    /// path to the plumber API (default `plumber.R`)
    #[argh(option, short = 'f', default = r#"String::from("plumber.R")"#)]
    file: String,

    /// default 10. Interval in seconds when to check for unused connections
    #[argh(option, default = "10")]
    check_unused: u32,

    /// default 5 mins. How long an API can go unused before being killed in seconds.
    #[argh(option, default = "300")]
    max_age: u32,
}

//#[tokio::main(worker_threads = 5)]
fn main() {
    let cli_args: Cli = argh::from_env();

    // validate that the file exists
    let p = std::path::Path::new(&cli_args.file).try_exists().unwrap();
    if !p {
        panic!("plumber file does not exist.")
    }

    if cli_args.n_max < 1 {
        panic!("Cannot have fewer than 1 plumber API")
    }

    if cli_args.workers < 1 {
        panic!("Cannot have fewer than 1 worker thread")
    }

    // verify that R is installed will panic if R isn't found (i hope)
    std::process::Command::new("R")
        .arg("--version")
        .stdout(Stdio::null())
        .spawn()
        .unwrap();

    // verify that the plumber pcackage can be found
    let plumber_exists = std::process::Command::new("Rscript")
        .arg("-e")
        .arg("library(plumber)")
        .stderr(Stdio::piped())
        .output()
        .unwrap()
        .status
        .success();

    // panic if plumber isn't found
    if !plumber_exists {
        panic!("plumber package cannot be found")
    }

    // build the tokio runtime
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(cli_args.workers as usize)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            start::valve_start(
                cli_args.file,
                cli_args.host,
                cli_args.port,
                cli_args.n_max.into(),
                cli_args.check_unused.try_into().unwrap(),
                cli_args.max_age.try_into().unwrap(),
            )
            .await;
        })
}
