# Valve

`valve` creates multi-threaded [plumber APIs](https://www.rplumber.io/) powered by Rust's [tokio](https://github.com/tokio-rs/tokio) and [axum](https://github.com/tokio-rs/axum) web frameworks.

## Motivation

Plumber is an R package that creates RESTful APIs from R functions. It is limited in that each API is a single R process and thus a single thread. Multiple queries are executed in the sequence that they came in. Scaling plumber APIs is not easy. The intention of valve is to be able to make scaling plumber APIs, and thus R itself, easier. This is done by spawning plumber APIs on multiple threads and having an axum app on a main thread distributing requests across the multiple plumber APIs. We can make R better by leveraging Rust's ["fearless concurrency."](https://blog.rust-lang.org/2015/04/10/Fearless-Concurrency.html)


## Installation

Install the R package using {remotes}. Note that this will compile the package from source which will require rust to be installed. If you don't have rust installed follow the instructions [here](https://www.rust-lang.org/tools/install). Rust is the second easiest programming language to install after R. 

> I also recommend installing the development version of {rextendr} via `pak::pak("extendr/rextendr")` which provides the function `rextendr::rust_sitrep()` which will update you on if you have a compatible Rust installation.

```r
remotes::install_github("josiahparry/valve")
```

When the R package is built it also includes the binary executable in the `inst/` folder. 

To install the executable only run

```bash
cargo install --git https://github.com/josiahparry/valve/
```

## Creating the app

The R package exports only 1 function: `valve_run()`. It takes 5 arguments, the path to the plumber API (not that the `dir` argument is not used and only `file` is), the host, port, number of threads, and number of workers. The host and port determine _where_ your valve app will run and the last two arguments `n_threads` and `workers` determines how requests will be scaled. 

`n_threads` refers to how many background Plumber API processes with be spawned whereas the `workers` argument determines how many asynchronous worker threads are created by tokio. Generally, the number of `workers` should be equal to the number of plumber APIs since because plumber is single threaded. Also note that connections are automatically pooled by [hyper](https://docs.rs/hyper/latest/hyper/client/index.html).

```r
library(valve)
# get included plumber API path
plumber_api_path <- system.file("plumber.R", package = "valve")

valve_run(plumber_api_path, n_threads = 5, workers = 5)
#> Docs hosted at <http://127.0.0.1:3000/__docs__/>
#> Spawning Plumber API at 127.0.0.1:13299
#> Spawning Plumber API at 127.0.0.1:10285
#> Spawning Plumber API at 127.0.0.1:24348
#> Spawning Plumber API at 127.0.0.1:32704
#> Spawning Plumber API at 127.0.0.1:23204
```

Running this from your R session will block the session. If you are comfortable, it is recommended to install the cli so you can run them from your terminal so that you can call the plumber APIs from your R session.

```bash
# the same as the above but from the terminal
valve -f $(Rscript -e "cat(system.file('plumber.R', package = 'valve'))") -n 5 -w 5
```

## Calling valve with multiple workers

The way valve works is by accepting requests on a main port (3000 by default) and then distributing the requests round robin to the plumber APIs that are spawned on random ports. Requests are captured by `axum` and proxied to a plumber API process.

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

## Benchmarks with drill

Simple benchmarks using drill can be found in `inst/bench-sleep-plumber.yml` and `bench-sleep-valve.yml`. 

The bench mark calls the `/sleep` endpoint and sleeps for 500ms for 100 times with 5 concurrent threads. This alone can illustrate how much we can speed up a single plumber API's response time with valve.

Plumber's benchmark:

```
Time taken for tests      50.7 seconds
Total requests            100
Successful requests       100
Failed requests           0
Requests per second       1.97 [#/sec]
Median time per request   2540ms
Average time per request  2482ms
Sample standard deviation 272ms
99.0'th percentile        2556ms
99.5'th percentile        2556ms
99.9'th percentile        2556ms
```


Valve's benchmark: 

```
Time taken for tests      10.2 seconds
Total requests            100
Successful requests       100
Failed requests           0
Requests per second       9.78 [#/sec]
Median time per request   510ms
Average time per request  510ms
Sample standard deviation 2ms
99.0'th percentile        516ms
99.5'th percentile        518ms
99.9'th percentile        518ms
```

### With all that said....

valve is best suited for light-ish work loads. Each background plumber API will hold their own copy of their R objects. So if you are serving a machine learning model that is a GB big, that model will have to be copied into each thread and that can be quickly bloat up your ram. So be smart! If you have massive objects in your R session, try and reduce the clutter and thin it out. 
