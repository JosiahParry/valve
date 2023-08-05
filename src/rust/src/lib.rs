pub mod plumber;
pub mod start;

pub use plumber::*;
pub use start::*;

#[cfg(feature = "rlib")]
use extendr_api::prelude::*;

#[cfg(feature = "rlib")]
#[extendr]
pub fn valve_run_(
    filepath: String,
    host: String,
    port: u16,
    workers: u16,
    n_min: u16,
    n_max: u16,
    check_interval: i32,
    max_age: i32,
) {
    let workers = workers as usize;
    let n_min = n_min as usize;
    let n_max = n_max as usize;
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(workers)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            tokio::select! {
                _ = valve_start(filepath, host, port, n_min, n_max, check_interval, max_age) => {
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

#[cfg(feature = "rlib")]
extendr_module! {
    mod valve;
    fn valve_run_;
}
