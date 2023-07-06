use crate::shiny::*;

use hyper::client::HttpConnector;

type Client = hyper::client::Client<HttpConnector, Body>;

use axum::{body::Body, extract::Extension};

use std::time::Duration;

use std::sync::Arc;

use deadpool::managed;
type Pool = managed::Pool<ShinyManager>;

pub async fn valve_start_shiny_(
    app_dir: String,
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

    let app_dir = Arc::new(app_dir);
    let axum_host = Arc::new(host);
    let axum_port = port;

    // spawn client used for proxying
    let c = Client::new();

    // create Pool manager
    let manager = ShinyManager {
        host: axum_host.to_string(),
        app_dir: app_dir.to_string(),
    };

    // Build the Plumber API connection Pool
    let pool = Pool::builder(manager).max_size(n_max).build().unwrap();

    // define the APP
    let app = axum::Router::new()
        .route("/", axum::routing::any(shiny_handler))
        //.route("/*key", axum::routing::any(shiny_handler))
        .with_state(c)
        .layer(Extension(pool.clone()));

    // This thread is used to check if there are expired threads
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(interval).await;
            pool.retain(|pr, metrics| {
                let too_old = metrics.last_used() < max_age;

                if !too_old {
                    println!("Killing Shiny App at {}:{}", pr.host, pr.port);
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
