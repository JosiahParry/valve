use crate::pooling::*;

use hyper::{client::HttpConnector};
use rand::Rng;
type Client = hyper::client::Client<HttpConnector, Body>;

use axum::{
    body::Body,
    extract::{Extension, State},
    http::Request,
    response::{Redirect, Response},
    routing::get,
};

use std::time::Duration;

use std::{
    net::TcpListener,
    process::{Command, Stdio},
    sync::{Arc},
};

pub async fn valve_start(filepath: String, host: String, port: u16, _n_max: usize) {
    let filepath = Arc::new(filepath);
    let axum_host = Arc::new(host);
    let axum_port = port;


    // spawn client used for proxying
    let c = Client::new();


    let plumber_manager = PrManager { 
        host: axum_host.to_string(), 
        pr_file: filepath.to_string() 
    };


    let pool = Pool::builder(plumber_manager)
        //.max_size(value)
        //.timeouts(Timeouts::new())
        //.wait_timeout(1000 * 10)
        //.timeouts(60)
        .build()
        .unwrap();



    let app = axum::Router::new()
        .route("/", get(|| async { Redirect::permanent("/__docs__/") }))
        .route("/*key", axum::routing::any(plumber_handler))
        .with_state(c)
        .layer(Extension(pool.clone()));


    // determines how often to check connects
    let interval = Duration::from_secs(5);
    // determines how old a connection can be before being killed
    let max_age = Duration::from_secs(10);

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(interval).await;
            pool.retain(|pr, metrics| {
                let too_old = metrics.last_used() < max_age;

                if !too_old {
                    println!("Killing plumber API at {}:{}", pr.host, pr.port);
                }

                too_old
            });
        }
    });

    // Start the Axum server
    let full_axum_host = format!("{axum_host}:{axum_port}");
    axum::Server::try_bind(&full_axum_host.as_str().parse().unwrap())
        .unwrap()
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// spawn plumber
use std::io::{BufReader, BufRead};
use std::process::Child;
pub fn spawn_plumber(host: &str, port: u16, filepath: &str) -> Child {
    let mut pr_child = Command::new("R")
        .arg("-e")
        .arg(format!(
            "plumber::plumb('{filepath}')$run(host = '{host}', port = {port})"
        ))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start R process");

    let stdout = pr_child.stderr.take().expect("stdout to be read");
    let reader = BufReader::new(stdout);
    
    for line in reader.lines() {
        if let Ok(line) = line {
            //println!("{}", line);
            if line.contains("Running swagger") {
                std::thread::sleep(Duration::from_millis(100));
                //println!("plumber started");
                break;
            }
        }
    }

    pr_child
    
}

// from chatGPT
// these functions generate random
pub fn generate_random_port(host: &str) -> u16 {
    let mut rng = rand::thread_rng();
    loop {
        let port: u16 = rng.gen_range(1024..=65535);
        if is_port_available(host, port) {
            return port;
        }
    }
}
// checks to see if the port is available
fn is_port_available(host: &str, port: u16) -> bool {
    match TcpListener::bind(format!("{host}:{port}")) {
        Ok(listener) => {
            // The port is available, so we close the listener and return true
            drop(listener);
            true
        }
        Err(_) => false, // The port is not available
    }
}


use deadpool::managed;
type Pool = managed::Pool<PrManager>;

async fn plumber_handler(
    State(client): State<Client>,
    Extension(pr_pool): Extension<Pool>,
    req: Request<Body>
) -> Response {

    pr_pool.get()
        .await
        .unwrap()
        .proxy_request(client, req).await

}
// It appears that what i need is a reverse proxy
// tokio discord directed me to https://github.com/tokio-rs/axum/blob/v0.6.x/examples/reverse-proxy/src/main.rs
// which is an example of this.
// essentially a reverse proxy takes a request, sends, it and returns it (what i want)
// this requires a client to make the request. so if thats the case, i may want to
// have a single client spawned for each port (plumber API) and this can be part
// of the state it can be a struct like struct Plumber{ port = u16, client = Client };
