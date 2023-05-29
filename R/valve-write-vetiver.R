

#' Write a Dockerfile for vetiver using valve
#'
#' This function is a modification of `vetiver::vetiver_write_docker()` which utilizes
#' valve to run model prediction endpoints in parallel.
#'
#' @inheritParams vetiver::vetiver_write_docker
#' @param vetiver_args additional arguments passed to `vetiver::vetiver_write_docker()` as
#'  key-value pairs in a list object.
#'
#' @returns The content of the Dockerfile, invisibly.
#'
valve_write_vetiver <- function(vetiver_model,
                                plumber_file = "plumber.R",
                                path = ".",
                                port = 8000,
                                vetiver_args = list()
                                # valve_opts = c("-n = 5", "-w = 5")
                                ) {

  if (!requireNamespace("vetiver")) {
    stop("`vetiver` package must be installed")
  }


  all_args <- c(
    vetiver_args,
    list(
      vetiver_model = vetiver_model,
      plumber_file = plumber_file,
      path = path,
      port = port
    )
  )

  # get original docker file contents
  dock <- do.call(vetiver_write_docker, all_args)

  # commands for installing rust
  install_rust <- c(
    "RUN apt-get install -y -q \\
    build-essential \\
    curl\n",
    'RUN curl https://sh.rustup.rs -sSf | sh -s -- -y',
    'ENV PATH="/root/.cargo/bin:${PATH}"'
  )

  # command for installing valve
  install_valve <- c(
    "RUN cargo install --git https://github.com/josiahparry/valve/ --no-default-features\n"
  )


  # if the port is a character creating a command to be executed in its place
  if (is.character(port)) {
    port <- paste0(c("$(Rscript -e 'cat(", port, ")')"), collapse = "")
  }

  # TO DO add valve options
  # entrypoint command
  entrypoint <- paste0("ENTRYPOINT valve -f /opt/ml/plumber.R -h 0.0.0.0 -p ", port)

  to_append <- c(install_rust, install_valve, entrypoint)

  dock[[length(dock)]] <- to_append

  invisible(unlist(dock))

}

