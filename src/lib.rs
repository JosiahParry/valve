mod start;
use crate::start::valve_start;

fn valve_create(host: String, port: u16, n_threads: u16, workers: usize) {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(workers)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            valve_start(host, port, n_threads).await;
        })

}
