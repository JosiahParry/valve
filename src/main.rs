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


#[tokio::main(worker_threads = 5)]
async fn main() {
    // specify the number of Plumber APIs to spawn
    let num_threads = 5;
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
        let ports_clone = Arc::clone(&ports);

        let _handle = thread::spawn(move || {
            let port = ports_clone.lock().unwrap().next().unwrap();
            println!("Spawning thread on port {port}");
            spawn_plumber(port);
        });
    }

    // Access the ports data
    //let ports_data = ports.clone();
    println!("Spawned ports: {ports:?}");

    // first port will be used to host docs
    let first_port = ports.lock().unwrap().next().unwrap();

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
                async move {
                    let port = ports.lock().unwrap().next().unwrap();
                    let ruri = req.uri();
                    let mut uri = ruri.clone().into_parts();
                    uri.authority = Some(format!("127.0.0.1:{port}").as_str().parse().unwrap());
                    uri.scheme = Some("http".parse().unwrap());
                    let new_uri = Uri::from_parts(uri).unwrap().to_string();
                    Redirect::temporary(new_uri.as_str())
                }
            }),
        );

    // Start the Axum server
    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}



// spawn plumber
fn spawn_plumber(port: u16) {
    let mut _output = Command::new("R")
        .arg("-e")
        .arg(format!("plumber::plumb('plumber.R')$run(port = {port})"))
        .stdin(Stdio::null())
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
