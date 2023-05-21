use axum::response::Redirect;
use std::thread;
//use extendr_engine::*;
//use extendr_api::prelude::*;
//use reqwest::blocking::Client;

use std::process::Command;
use std::process::Stdio;
use std::sync::{Arc, Mutex};

use axum::{body::Body, routing::{get, post}};

#[tokio::main]
async fn main() {
    // specify the number of Plumber APIs to spawn
    let num_threads = 5;
    let ports = Arc::new(Mutex::new(Vec::<u16>::new()));

    // start R and print R_HOME
    // All threads that are spawned for a plumber API are going to be blocked
    // Those threads can't ever be used to return a value. So joining is impossible
    for _ in 0..num_threads {
        let ports_clone = Arc::clone(&ports);
        let _handle = thread::spawn(move || {
            let port = generate_random_port();
            println!("Spawning thread on port {port}");
            spawn_plumber(port, ports_clone);
        });
    }

    // i need to sleep to wait for the R to start and to start the plumber API
    // there needs to be a better way to do this than waiting some number of secs
    thread::sleep(core::time::Duration::from_secs(2));

    // Access the ports data
    let ports_data = ports.lock().unwrap().clone();
    println!("Spawned ports: {ports_data:?}");

    // first port will be used to host docs
    let first_port = ports_data[0];

    // Create the Axum application
    let app = axum::Router::new()
        .route(
            "/__docs__",
            get(move || {

                async move {
                    // Create the docs path using the cloned value
                    let doc_path = format!("http://127.0.0.1:{first_port}/__docs__/");
                    Redirect::to(doc_path.as_str())
                }
            }),
        )
        .route(
            "/*key",
            axum::routing::any(move |req: Request<Body>| {
                // grab random port
                //let pts = ports.lock().unwrap();
                let port = get_random_plumber_port(&ports.lock().unwrap());

                async move {
                    let ruri = req.uri();
                    let mut uri = ruri.clone().into_parts();
                    uri.authority = Some(format!("127.0.0.1:{port}").as_str().parse().unwrap());
                    uri.scheme = Some("http".parse().unwrap());
                    println!("{uri:?}");
                    let new_uri = Uri::from_parts(uri).unwrap().to_string();
                    Redirect::temporary(new_uri.as_str())
                }
            }),
        )
        ;

    // Start the Axum server
    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

use axum::{
    http::{Request},
};

use hyper::Uri;

// spawn plumber
fn spawn_plumber(port: u16, ports: Arc<Mutex<Vec<u16>>>) {
    ports.lock().unwrap().push(port);
    let mut _output = Command::new("R")
        .arg("-e")
        .arg(format!("plumber::plumb('plumber.R')$run(port = {port})"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start R process");
}

use std::sync::MutexGuard;
// gets a single port number from available plumber ports
fn get_random_plumber_port(ports: &MutexGuard<Vec<u16>>) -> u16 {
    let mut rng = rand::thread_rng();
    //let ports_data = ports.lock().unwrap();
    let index = rng.gen_range(0..ports.len());
    ports[index]
}

// from chatGPT
// these functions generate random
use rand::Rng;
use std::net::TcpListener;
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
    match TcpListener::bind(format!("127.0.0.1:{port}")) {
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

