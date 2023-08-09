# stage 1: install valve
FROM rust:slim AS builder
RUN cargo install valve-rs --no-default-features

# stage 2: configure the container to run plumber
FROM rocker/r-ver:4.3.0

ENV VALVE_HOST 0.0.0.0
ENV VALVE_PORT 8000

# always use RSPM 
ENV RENV_CONFIG_REPOS_OVERRIDE https://packagemanager.rstudio.com/cran/latest

RUN apt-get update -qq && apt-get install -y --no-install-recommends \
  libcurl4-openssl-dev \
  libicu-dev \
  libsodium-dev \
  libssl-dev \
  make \
  zlib1g-dev \
  && apt-get clean

COPY renv.lock renv.lock
RUN Rscript -e "install.packages('renv')"
RUN Rscript -e "renv::restore()"
COPY plumber.R /api/plumber.R

# get binary from builder 
COPY --from=builder /usr/local/cargo/bin/valve /usr/local/bin/

# open the port for valve
EXPOSE ${VALVE_PORT}

# Start Valve app
ENTRYPOINT valve -f /api/plumber.R -h $VALVE_HOST -p $VALVE_PORT --workers 10 --n-max 10 --check-unused 10 --max-age 300
