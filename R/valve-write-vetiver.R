

#' Write a Dockerfile for vetiver using valve
#'
#' A Valve powered {vetiver} Dockerfile will run prediction endpoints concurrently.
#'
#' This function is a modification of `vetiver::vetiver_write_docker()`. It
#' modifies the created Dockerfile to install Valve, changes the `ENTRYPOINT` to
#' use the Valve executable instead of a single plumber API via an R command.
#'

#'
#' @inheritParams vetiver::vetiver_write_docker
#' @param vetiver_args additional arguments passed to `vetiver::vetiver_write_docker()` as
#'  key-value pairs in a list object.
#'
#' @returns The content of the Dockerfile, invisibly.
#'
#' @export
valve_write_vetiver <- function(vetiver_model,
                                plumber_file = "plumber.R",
                                path = ".",
                                port = 8000,
                                vetiver_args = list(),
                                workers = NULL,
                                n_max = NULL,
                                check_unused = NULL,
                                max_age = NULL
                                ) {

  if (!requireNamespace("vetiver")) {
    stop("`vetiver` package must be installed")
  }

  # make sure its a vetiver model
  # type checks on args to valve
  stopifnot(
    inherits(vetiver_model, "vetiver_model"),
    "`workers` must be a numeric scalar" = is.numeric(workers) && length(workers) == 1,
    "`n_max` must be a numeric scalar" = is.numeric(n_max) && length(n_max) == 1,
    "`check_unused` must be a numeric scalar" = is.numeric(check_unused) && length(check_unused) == 1,
    "`max_age` must be a numeric scalar" = is.numeric(max_age) && length(max_age) == 1
  )

  # identify additional valve arguments provided and tidy them up
  addtl_valve_opts <- c(
    workers = workers,
    n_max = n_max,
    check_unused = check_unused,
    max_age = max_age
  )

  # change names to the cli flag names
  arg_flags <- paste("--", gsub("_", "-", names(addtl_valve_opts)), sep = "")

  # get names and values appropriately
  valve_opts <- paste(arg_flags,  addtl_valve_opts, collapse = " ")


  all_vetiver_args <- c(
    vetiver_args,
    list(
      vetiver_model = vetiver_model,
      plumber_file = plumber_file,
      path = path,
      port = port
    )
  )

  # get original docker file contents
  dock <- do.call(vetiver_write_docker, all_vetiver_args)

  # commands for installing rust
  install_rust <- c(
    "\n# Install Rust toolchain & add to the path",
    "RUN apt-get install -y -q \\
    build-essential \\
    curl  ",
    'RUN curl https://sh.rustup.rs -sSf | sh -s -- -y',
    'ENV PATH="/root/.cargo/bin:${PATH}"'
  )

  # command for installing valve
  install_valve <- c(
    "\n# Install Valve binary from Github",
    "RUN cargo install valve-rs --no-default-features"
  )


  # if the port is a character creating a command to be executed in its place
  if (is.character(port)) {
    port <- paste0(c("$(Rscript -e 'cat(", port, ")')"), collapse = "")
  }

  # entrypoint command
  entrypoint <- paste0(
    "ENTRYPOINT valve -f /opt/ml/plumber.R -h 0.0.0.0 -p ",
    port,
    " ",
    valve_opts
  )

  to_append <- c(install_rust, install_valve, "\n# Start Valve app", entrypoint)

  # remove the entrypoint
  dock_vec <- unlist(dock[1:(length(dock)-1)])

  # append custom commands
  dock_vec <- c(dock_vec, to_append)

  # writeover Dockerfile
  writeLines(dock_vec, file.path(path, "Dockerfile"))

  invisible(dock_vec)

}

