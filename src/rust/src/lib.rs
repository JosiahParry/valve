pub mod start;
use extendr_api::prelude::*;

use crate::start::valve_start;

#[extendr]
pub fn valve_run_(filepath: String, host: String, port: u16, n_threads: u16, workers: u16) {
    let workers = workers as usize;
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(workers)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            tokio::select! {
                _ = valve_start(filepath, host, port, n_threads) => {
                }
                r = tokio::signal::ctrl_c() => {
                    match r {
                        Ok(()) => {/* cancelled */}
                        Err(e) => eprintln!("Unable to listen for shutdown signal: {e}"),
                    }
                }
            };
        });
}

extendr_module! {
    mod valve;
    fn valve_run_;
}
