sleep <- function(port, secs) {
  httr2::request(
    paste0("127.0.0.1:", port, "/sleep?zzz=", secs)
  ) |>
    httr2::req_perform() |>
    httr2::resp_body_string()
}


library(valve)
plumber_api_path <- system.file("plumber.R", package = "valve")

valve_run(plumber_api_path, n_max = 10)

library(furrr)
plan(multisession, workers = 5)

start <- Sys.time()
multi_sleep <- future_map(1:5, ~ sleep(3000, 2))
multi_total <- Sys.time() - start

start <- Sys.time()
single_sleep <- furrr::future_map(1:5, ~ sleep(5474, 2))
single_total <- Sys.time() - start
