use crate::plumber::*;

use hyper::client::HttpConnector;

type Client = hyper::client::Client<HttpConnector, Body>;

use axum::{body::Body, extract::Extension, response::Redirect, routing::get};

use std::time::Duration;

use std::sync::Arc;

use deadpool::managed;
type Pool = managed::Pool<PrManager>;

pub async fn valve_start_plumber_(
    filepath: String,
    host: String,
    port: u16,
    n_max: usize,
    check_interval: i32,
    max_age: i32,
) {
    // determines how often to check connects
    let interval = Duration::from_secs(check_interval.try_into().unwrap());
    // determines how old a connection can be before being killed
    let max_age = Duration::from_secs(max_age.try_into().unwrap());

    let filepath = Arc::new(filepath);
    let axum_host = Arc::new(host);
    let axum_port = port;

    // spawn client used for proxying
    let c = Client::new();

    // create Pool manager
    let plumber_manager = PrManager {
        host: axum_host.to_string(),
        pr_file: filepath.to_string(),
    };

    // Build the Plumber API connection Pool
    let pool = Pool::builder(plumber_manager)
        .max_size(n_max)
        .build()
        .unwrap();

    // define the APP
    let app = axum::Router::new()
        .route("/", get(|| async { Redirect::permanent("/__docs__/") }))
        .route("/*key", axum::routing::any(plumber_handler))
        .with_state(c)
        .layer(Extension(pool.clone()));

    // This thread is used to check if there are expired threads
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
