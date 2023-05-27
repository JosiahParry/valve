use hyper::{client::HttpConnector, Uri};
use rand::Rng;
type Client = hyper::client::Client<HttpConnector, Body>;

use axum::{
    body::Body,
    extract::{Extension, State},
    http::Request,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};

use std::{
    iter::Cycle,
    net::TcpListener,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};

pub async fn valve_start(filepath: String, host: String, port: u16, n_threads: u16) {
    let filepath = Arc::new(filepath);
    let axum_host = Arc::new(host);
    let axum_port = port;
    let n_threads = n_threads;

    // spawn client
    let c = Client::new();

    // specify the number of Plumber APIs to spawn
    let num_threads = n_threads; // create the iterator for ports
    let ports: Arc<Mutex<Cycle<std::vec::IntoIter<u16>>>> = Arc::new(Mutex::new(
        (0..num_threads)
            .map(|_| generate_random_port(axum_host.as_str()))
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
        let axum_host = axum_host.clone();
        let fp = filepath.clone();
        let _handle = thread::spawn(move || {
            let port = port_i;
            println!("Spawning Plumber API at {axum_host}:{port}");
            spawn_plumber(&axum_host, port, fp.as_str());
        });
    }

    let app = axum::Router::new()
        .route("/", get(|| async { Redirect::permanent("/__docs__/") }))
        .route("/*key", axum::routing::any(redir_handler))
        .with_state(c)
        .layer(Extension(axum_host.clone()))
        .layer(Extension(ports));

    // Start the Axum server
    let full_axum_host = format!("{axum_host}:{axum_port}");
    axum::Server::try_bind(&full_axum_host.as_str().parse().unwrap())
        .unwrap()
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// spawn plumber
fn spawn_plumber(host: &str, port: u16, filepath: &str) {
    let mut _output = Command::new("R")
        .arg("-e")
        .arg(format!(
            "plumber::plumb('{filepath}')$run(host = '{host}', port = {port})"
        ))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start R process");
}

// from chatGPT
// these functions generate random
fn generate_random_port(host: &str) -> u16 {
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

async fn redir_handler(
    State(client): State<Client>,
    Extension(host): Extension<Arc<String>>,
    Extension(ports): Extension<Arc<Mutex<Cycle<std::vec::IntoIter<u16>>>>>,
    mut req: Request<Body>,
) -> Response {
    // select the next port
    let port = ports.lock().unwrap().next().unwrap();
    let ruri = req.uri(); // get the URI
    let mut uri = ruri.clone().into_parts(); // clone
                                             // change URI to random port from above
    uri.authority = Some(
        format!("{}:{port}", host.as_str())
            .as_str()
            .parse()
            .unwrap(),
    );
    // TODO enable https or other schemes
    uri.scheme = Some("http".parse().unwrap());

    *req.uri_mut() = Uri::from_parts(uri).unwrap();

    client.request(req).await.unwrap().into_response()
}

// It appears that what i need is a reverse proxy
// tokio discord directed me to https://github.com/tokio-rs/axum/blob/v0.6.x/examples/reverse-proxy/src/main.rs
// which is an example of this.
// essentially a reverse proxy takes a request, sends, it and returns it (what i want)
// this requires a client to make the request. so if thats the case, i may want to
// have a single client spawned for each port (plumber API) and this can be part
// of the state it can be a struct like struct Plumber{ port = u16, client = Client };
