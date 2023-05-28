

use crate::start::{generate_random_port, spawn_plumber};
// use std::{
//     sync::{Arc, Mutex},
//     iter::Cycle,
//     vec::IntoIter
// };

// for manager struct
use async_trait::async_trait;
use deadpool::managed;

use axum::{
    body::Body,
    http::Request,
    response::{IntoResponse, Response}
};

use hyper::{client::HttpConnector, Uri};
type Client = hyper::client::Client<HttpConnector, Body>;


pub struct Plumber {
    pub host: String, 
    pub port: u16,  
    pub process: std::process::Child
}

impl Plumber {
    pub fn spawn(host: &str, filepath: &str) -> Self {
        let port = generate_random_port(host);
        let process = spawn_plumber(host, port, filepath);
        println!("Spawning plumber API at {}:{}", host, port);
        Self { host: host.to_string(), port, process }   
    }

    pub fn is_alive(&mut self) -> bool {
        let status = self.process.try_wait();
        match status {
            Ok(Some(_)) => true,
            Ok(None) => false,
            Err(_) => false
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

         // TODO enable https or other schemes
        uri.scheme = Some("http".parse().unwrap());
        *req.uri_mut() = Uri::from_parts(uri).unwrap();
        client.request(req).await.unwrap().into_response()
    }
}

// This struct will contain the iterator that is used in the axum
// app to cycle through ports. though that might not be necessary
// since the Plumber struct contains the port
// the plumber struct will be returned by the pool and 
// can be used in the axum route directly


pub struct PrManager {
//    ports: Arc<Mutex<Cycle<std::vec::IntoIter<u16>>>>
    pub host: String,
    pub pr_file: String
}

#[derive(Debug)]
pub enum Error { Fail }

#[async_trait]
impl managed::Manager for PrManager {
    type Type = Plumber;
    type Error = Error;

    async fn create(&self) -> Result<Plumber, Error> {
        let host = self.host.as_str();
        let filepath = self.pr_file.as_str();
        Ok(Plumber::spawn(host, filepath))
    }

    async fn recycle(&self, _conn: &mut Plumber) -> managed::RecycleResult<Error> {
        Ok(())
    }

    fn detach(&self, obj: &mut Plumber) {
        let _killed_process = obj.process.kill();

    }
 }

