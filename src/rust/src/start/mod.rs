pub mod start_plumber;
pub use start_plumber::*;

pub mod start_shiny;
pub use start_shiny::*;

use rand::Rng;
use std::net::TcpListener;
// from chatGPT
// these functions generate random ports and
// check if they are in use
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
