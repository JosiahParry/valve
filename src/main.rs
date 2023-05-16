use std::thread;
use extendr_engine::*;
use extendr_api::prelude::*;
use reqwest::blocking::Client;

fn main() {

    // start R and print R_HOME

    let _t1 = thread::spawn(move || {
        start_r();
        let r_home = std::env::var("R_HOME").unwrap(); 
        println!("R_HOME {r_home}");
        // Start the plumber API 
        //let port = 8888_i32;
        let _plumb_spawn = R!(r#"plumber::pr_run(plumber::plumb("plumber.R"), port = 8888)"#); 
    });

    // i need to sleep to wait for the R to start and to start the plumber API
    // there needs to be a better way to do this than waiting some number of secs

    thread::sleep(core::time::Duration::from_secs(1));
    
   let url = "http://127.0.0.1:8888/echo?msg=hello-extendr";
    
    let client = Client::new();
    let resp = client.get(url).send().unwrap();
    
    println!("\n{}\n", resp.text().unwrap());
    println!("R has been terminated");

}

