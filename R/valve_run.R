#' Start a valve App
#'
#' Run a plumber API in parallel using valve. Plumber APIs are spawned on
#' `n_threads` asynchronous threads on random ports. Incoming requests are handled
#' on a single port and redirected to the plumber APIs in a
#' simple round-robin fashion.
#'
#' @param filepath default `"plumber.R"`. The path to the Plumber API. Provided to
#'    the `file` argument of `plumber::plumb()`.
#' @param host default `"127.0.0.1"`. Where to host the valve app and Plumber APIs.
#' @param port default `3000`. The port to host the valve app on.
#' @param n_threads default `3`. The number of Plumber APIs to run in parallel.
#' @param workers default `3`. The number of worker threads in the valve app to
#'    execute requests. This number should not be larger than `n_threads + 1`.
valve_run <- function(filepath = "plumber.R",
                      host = "127.0.0.1",
                      port = 3000,
                      n_threads = 3,
                      workers = 4) {

  fp <- filepath
  hst <- host
  prt <- port
  n <- n_threads
  wrks <- workers

  docs <- paste0("http://", hst, ":", prt, "/__docs__")
  cli::cli_inform("Docs hosted at {.url {docs}}")
  valve_run_(fp, hst, prt, n, wrks)

}


