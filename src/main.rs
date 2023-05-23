//mod start;
//use crate::start::valve_start;
use valve::start::valve_start;

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
}

//#[tokio::main(worker_threads = 5)]
fn main() {

    let cli_args: Cli = argh::from_env();
    println!("{cli_args:?}");

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            valve_start(cli_args.host, cli_args.port, cli_args.n_threads).await;
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