use valve::start;
use std::process::Stdio;

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

    /// number of plumber APIs to spawn
    #[argh(option, short = 'n', default = "3")]
    n_threads: u16,

    /// number of Tokio workers to spawn
    #[argh(option, short = 'w', default = "3")]
    workers: u16,

    /// path to the plumber API (default `plumber.R`)
    #[argh(option, short = 'f', default = r#"String::from("plumber.R")"#)]
    file: String,
}

//#[tokio::main(worker_threads = 5)]
fn main() {
    let cli_args: Cli = argh::from_env();

    // validate that the file exists
    let p = std::path::Path::new(&cli_args.file).try_exists().unwrap();
    if !p {
        panic!("plumber file does not exist.")
    }

    if cli_args.n_threads < 1 {
        panic!("Cannot have fewer than 1 plumber API")
    }

    if cli_args.workers < 1 {
        panic!("Cannot have fewer than 1 worker thread")
    }

    // run R --version if error things will panic
    std::process::Command::new("R")
        .arg("--version")
        .stdout(Stdio::null())
        .spawn()
        .unwrap();

    let plumber_exists = std::process::Command::new("Rscript")
        .arg("-e")
        .arg("library(plumber)")
        .stderr(Stdio::piped())
        .output()
        .unwrap()
        .status
        .success();

    if !plumber_exists {
        panic!("plumber package cannot be found")
    }

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
                //cli_args.n_threads,
            )
            .await;
        })
}

// This approach does not work because eval_string / R! block the thread
// need a "detached child process"
// fn spawn_plumber(port: u16, ports: Arc<Mutex<Vec<u16>>>) {
//     start_r();
//     ports.lock().unwrap().push(port);
//     //let r_home = std::env::var("R_HOME").unwrap();
//     //println!("R_HOME {r_home}");
//     // Define R code to spawn the plumber API
//     let plumb_call = format!(r#"plumber::pr_run(plumber::plumb("plumber.R"), port = {})"#, &port);
//     // spawn the API
//     let _plumb_spawn = eval_string(plumb_call.as_str());
// }
