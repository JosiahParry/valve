use std::thread;
//use extendr_engine::*;
//use extendr_api::prelude::*;
use reqwest::blocking::Client;

use std::sync::{Arc, Mutex};
use std::process::{ Stdio};
use std::process::Command;

fn main() {

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
            println!("Spawning thread on port { }", port);
            spawn_plumber(port, ports_clone);
        });
    }

    // i need to sleep to wait for the R to start and to start the plumber API
    // there needs to be a better way to do this than waiting some number of secs
    thread::sleep(core::time::Duration::from_secs(2));

    // Access the ports data
    let ports_data = ports.lock().unwrap();
    println!("Spawned ports: {:?}", *ports_data);

    for port in ports_data.iter() {
        //format a simple request string
        println!("http://127.0.0.1:{port}/__docs__/");
        let url = format!("http://127.0.0.1:{}/echo?msg=hello-extendr from port no. {}", port, port);
         
         // spawn the client and send the request
         let client = Client::new();
         let resp = client.get(url).send().unwrap();
         
         println!("\n{}\n", resp.text().unwrap());
         println!("R has been terminated");

    }
    
    thread::sleep(core::time::Duration::from_secs(30));

}




fn spawn_plumber(port: u16, ports: Arc<Mutex<Vec<u16>>>) {
    ports.lock().unwrap().push(port);
    let mut _output = Command::new("R")
        .arg("-e")
        .arg(format!("plumber::plumb('plumber.R')$run(port = {})", port))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start R process");
}

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


// from chatGPT
// these functions generate random ports that are available
use rand::Rng;
use std::net::{TcpListener};

fn generate_random_port() -> u16 {
    let mut rng = rand::thread_rng();
    loop {
        let port: u16 = rng.gen_range(1024..=65535);
        if is_port_available(port) {
            return port;
        }
    }
}

fn is_port_available(port: u16) -> bool {
    match TcpListener::bind(format!("127.0.0.1:{}", port)) {
        Ok(listener) => {
            // The port is available, so we close the listener and return true
            drop(listener);
            true
        }
        Err(_) => false, // The port is not available
    }
}