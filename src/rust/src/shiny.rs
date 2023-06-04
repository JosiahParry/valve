use crate::start::generate_random_port;

// for manager struct
use async_trait::async_trait;
use deadpool::managed;

use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio, Child},
    time::Duration,
};

use axum::{
    body::Body,
    extract::{Extension, State},
    http::Request,
    response::{IntoResponse, Response},
};

use hyper::{client::HttpConnector, Uri};
type Client = hyper::client::Client<HttpConnector, Body>;

// Define the Plumber Struct
pub struct Shiny {
    pub host: String,
    pub port: u16,
    pub process: std::process::Child,
}



// Plumber methods for spawning, checking alive status and killing
impl Shiny {
    pub fn spawn(host: &str, app_dir: &str) -> Self {
        let port = generate_random_port(host);

        #[cfg(debug_assertions)]
        println!("about to spawn shiny");

        let process = spawn_shiny(host, port, app_dir);
        
//        #[cfg(debug_assertions)]
        println!("Spawning Shiny App at {host}:{port}");

        Self {
            host: host.to_string(),
            port,
            process,
        }
    }

    pub fn is_alive(&mut self) -> bool {
        let status = self.process.try_wait();
        match status {
            Ok(Some(_)) => true,
            Ok(None) => false,
            Err(_) => false,
        }
    }

    pub async fn proxy_request(&mut self, client: Client, mut req: Request<Body>) -> Response {
        let mut uri = req.uri().clone().into_parts(); // get the URI
                                                      //let host = self.host.as_str();
        uri.authority = Some(
            format!("{}:{}", self.host, self.port)
                .as_str()
                .parse()
                .unwrap(),
        );

        #[cfg(debug_assertions)]
        println!("about to proxy shiny");
        // TODO enable https or other schemes
        uri.scheme = Some("http".parse().unwrap());
        *req.uri_mut() = Uri::from_parts(uri).unwrap();
        client.request(req).await.unwrap().into_response()
    }
}


pub struct ShinyManager {
    //    ports: Arc<Mutex<Cycle<std::vec::IntoIter<u16>>>>
    pub host: String,
    pub app_dir: String,
}

use crate::plumber::Error;
#[async_trait]
impl managed::Manager for ShinyManager {
    type Type = Shiny;
    type Error = Error;

    async fn create(&self) -> Result<Shiny, Error> {
        let host = self.host.as_str();
        let app_dir = self.app_dir.as_str();
        Ok(Shiny::spawn(host, app_dir))
    }

    async fn recycle(&self, _conn: &mut Shiny) -> managed::RecycleResult<Error> {
        Ok(())
    }

    fn detach(&self, obj: &mut Shiny) {
        let _killed_process = obj.process.kill();
    }
}

// Might have to check requests to see what port they were coming from
// if that doesnt exist we will inject that into the header if it doesnt exist
// read it isf it does and redirect to the appropriate one that way state can be persisteted?
pub fn spawn_shiny(host: &str, port: u16, app_dir: &str) -> Child {
    // start the R processes
    let mut shiny_child = Command::new("R")
        .arg("-e")
        // the defines the R command that is used to start shiny
        .arg(format!(
            "shiny::runApp('{app_dir}', {port}, host = '{host}')"
        ))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start R process");

    #[cfg(debug_assertions)]
    println!("theoretically have spawned shiny");

    // capture stderr
    let stderr = shiny_child.stderr.take().expect("stdout to be read");
    let reader = BufReader::new(stderr);

    // read lines from buffer. when "Running swagger" is captured
    // then we sleep for 1/10th of a second to let the api start and continue
    for line in reader.lines().flatten() {
        if line.contains("Listening on") {
            std::thread::sleep(Duration::from_millis(100));
            break;
        }
    }

    shiny_child
}
