# get executable path and included api paths
valve_executable <- system.file("valve", package = "valve")
plumber_api_path <- system.file("plumber.R", package = "valve")

# check that they exist
file.exists(c(valve_executable, plumber_api_path))

valve::valve_run(plumber_api_path)

# run Valve from the R-package's executable
processx::run(
  valve_executable,
  args = c("-f", plumber_api_path),
  echo = TRUE
)


