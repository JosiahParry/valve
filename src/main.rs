use rand::Rng;
use hyper::Uri;

use axum::{
    http::Request,
    response::Redirect,
    body::Body, 
    routing::get,
};

use std::{
    thread,
    net::TcpListener,
    sync::{Arc, Mutex},
    process::{Command, Stdio}
};

// set constants 
const HOST: &str = "127.0.0.1";
//const AXUM_PORT: u16 = 3000;

use {argh::FromArgs, std::fmt::Debug};
#[derive(FromArgs, Debug)]
/// Reach new heights.
struct Cli {
    /// an optional nickname for the pilot
    #[argh(option, short = 'h', default = r#"String::from("127.0.0.1")"#)]
    host: String,

    /// an optional height
    #[argh(option, short = 'p', default = "3000")]
    port: u16,

    /// an optional direction which is "up" by default
    #[argh(option, short = 'n', default = "3")]
    n_threads: u16,
}
#[tokio::main(worker_threads = 5)]
async fn main() {


    let cli_args: Cli = argh::from_env();
    println!("{cli_args:?}");
    let host = cli_args.host.as_str();
    let AXUM_PORT = cli_args.port;
    let n_threads = cli_args.n_threads;
    // TODO spawn new threads if need be

    // specify the number of Plumber APIs to spawn
    let num_threads = n_threads;
    let ports = Arc::new(Mutex::new(
        (0..num_threads)
            .map(|_| generate_random_port())
            .collect::<Vec<u16>>()
            .into_iter()
            .cycle(),
    ));

    // start R and print R_HOME
    // All threads that are spawned for a plumber API are going to be blocked
    // Those threads can't ever be used to return a value. So joining is impossible
    for _ in 0..num_threads {
        //let ports_clone = Arc::clone(&ports);
        let port_i = ports.lock().unwrap().next().unwrap();


        let _handle = thread::spawn(move || {
            //let port = ports_clone.lock().unwrap().next().unwrap();
            //println!("{port_i}");
            let port = port_i;
            println!("Spawning Plumber API at {HOST}:{port}");
            spawn_plumber(HOST, port);
        });
    }

    // Access the ports data
    //let ports_data = ports.clone();
    println!("Spawned ports: {:?}", ports);

    // first port will be used to host docs
    let first_port = ports.clone().lock().unwrap().next().unwrap();

    // Create the Axum application
    let app = axum::Router::new()
        .route(
            "/__docs__",
            get(move || {
                async move {
                    // Create the docs path using the cloned value
                    let doc_path = format!("http://{HOST}:{first_port}/__docs__/");
                    Redirect::to(doc_path.as_str())
                }
            }),
        )
        .route(
            "/*key",
            axum::routing::any(move |req: Request<Body>| {
                async move {
                    // select the next port
                    let port = ports.lock().unwrap().next().unwrap();
                    let ruri = req.uri(); // get the URI
                    let mut uri = ruri.clone().into_parts(); // clone 
                    // change URI to random port from above
                    uri.authority = Some(format!("{HOST}:{port}").as_str().parse().unwrap());
                    // TODO enable https or other schemes
                    uri.scheme = Some("http".parse().unwrap());
                    let new_uri = Uri::from_parts(uri).unwrap().to_string();
                    Redirect::temporary(new_uri.as_str())
                }
            }),
        );

    // Start the Axum server
    let axum_host = format!("{HOST}:{AXUM_PORT}");
    axum::Server::bind(&axum_host.as_str().parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// spawn plumber
fn spawn_plumber(host: &str, port: u16) {
    let mut _output = Command::new("R")
        .arg("-e")
        .arg(format!("plumber::plumb('plumber.R')$run(host = '{host}', port = {port})"))
       // .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start R process");
}


// from chatGPT
// these functions generate random
fn generate_random_port() -> u16 {
    let mut rng = rand::thread_rng();
    loop {
        let port: u16 = rng.gen_range(1024..=65535);
        if is_port_available(port) {
            return port;
        }
    }
}
// checks to see if the port is available
fn is_port_available(port: u16) -> bool {
    match TcpListener::bind(format!("{HOST}:{port}")) {
        Ok(listener) => {
            // The port is available, so we close the listener and return true
            drop(listener);
            true
        }
        Err(_) => false, // The port is not available
    }
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