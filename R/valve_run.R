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
#' @param n_max default `3`. The maximum number of Plumber APIs to run in parallel.
#' @param workers default `n_max`. The number of worker threads in the valve app to
#'    execute requests. This number should typically mimic `n_max`.
#' @param check_unused default `10`. The time interval, in seconds, to check for
#'    unused connections.
#' @param max_age default `300` (five minutes). Specifies how long a connection
#'    can go unused without being terminated. If a connection reaches this age
#'    it will be terminated in the next pool check
#'    (interval determined by `check_unused`),
#' @import plumber
#' @export
#' @examples
#' if (interactive()) {
#'   plumber_api_path <- system.file("plumber.R", package = "valve")
#'   valve_run(plumber_api_path)
#' }
#'

valve_run <- function(filepath = "plumber.R",
                      host = "127.0.0.1",
                      port = 3000,
                      n_max = 3,
                      workers = n_max,
                      check_unused = 10,
                      max_age = 300
                      ) {

  stopifnot(
    "`n_max` cannot be fewer than 1" =  n_max > 1,
    "`workers` cannot be fewer than 1" =  workers > 1,
    "`max_age` cannot be less than `check_unused`" = check_unused < max_age,
    "plumber file cannot be found" = file.exists(filepath)
  )

  docs <- paste0("http://", host, ":", port)
  cat(paste0("Valve app hosted at \033]8;;", docs, "\a<", docs, ">\033]8;;\a\n"))
  valve_run_(filepath, host, port, n_max, workers, check_unused, max_age)

}

