# Valve

The purpose of `valve` is to create a multi-threaded plumber API utilizing `libR-sys` and the plumber R package.

Plumber is an R package that creates RESTful APIs from R functions. It is limited in that each API is a single R process and thus a single thread. The intention of this Rust crate is to be able to spawn multiple threads each with their own Plumber API leveraging Rust's "fearless concurrency."


## Installation

Clone this repository and navigate to the root. From the terminal execute

```bash
cargo install --path .
```

## Creating the app

To create a valve application use the newly installed binary. At present there are two options that can be modified: `port` and `n-threads`. 

By default valve will create 3 duplicate plumber APIs to distribute work across on the localhost (`127.0.0.1`) at port 3000. There is no upper limit on this as of right now on the number of plumber APIs so don't kill your session.

### Start the app

Run the following to start your application

```bash
valve -n 5
```

## Calling valve with multiple workers

The way valve works is by accepting requests on a main port (3000 by default) and then distributing the requests round robin to the plumber APIs that are spawned on random ports. Requests are captured by `axum` and redirected to a plumber API process.

First I'm going to define a function to call my `/sleep` endpoint. The function will take two parameters: the port and the duration of sleep. The port will be used to change between the valve app and a single plumber API.

```r
sleep <- function(port, secs) {
  httr2::request(
        paste0("127.0.0.1:", port, "/sleep?zzz=", secs)
    ) |> 
    httr2::req_perform() |> 
    httr2::resp_body_string()
}
```

Using this function we'll create 5 total R sessions each will make a request to sleep for 2 seconds.

``` r
library(furrr)
plan(multisession, workers = 5)
```

First, we'll ping the main valve app which will distribute requests round robin.

```r
start <- Sys.time()
multi_sleep <- future_map(1:5, ~ sleep(3000, 2))
multi_total <- Sys.time() - start
```

Next, we select only one of the available plumber APIs and query it. 

```r
start <- Sys.time()
single_sleep <- furrr::future_map(1:5, ~ sleep(35219, 2))
single_total <- Sys.time() - start
```
Notice the performance difference. 

```r
print(paste0("Multiple Plumber APIs: ", round(multi_total, 2)))
#> [1] "Multiple Plumber APIs: 2.63"
print(paste0("One Plumber API: ", round(single_total, 2)))
#> [1] "One Plumber API: 10.08"
```

In the former each worker gets to make the request in approximately the same amount of time. The latter has to wait for each subsequent step to finish before the next one can occur. So we've effectively distributed the work load. 

---------

## Design Idea

- a plumber API is defined by `plumber.R`
- `n` R processes are defined
    - each process spawns a plumber API at a different port
- Rust handles requests and directs them to an available plumber API
- Returns the curl response from the plumber API directly

