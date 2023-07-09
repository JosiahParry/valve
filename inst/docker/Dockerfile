FROM rocker/r-ver:4.3.0
ENV RENV_CONFIG_REPOS_OVERRIDE https://packagemanager.rstudio.com/cran/latest

ENV VALVE_HOST 0.0.0.0
ENV VALVE_PORT 8000

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

# Install Rust toolchain & add to the path
RUN apt-get install -y -q \
    build-essential \
    curl
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Valve binary from Github
RUN cargo install valve-rs --no-default-features

EXPOSE ${VALVE_PORT}

# Start Valve app
ENTRYPOINT valve -f /api/plumber.R -h $VALVE_HOST -p $VALVE_PORT --workers 10 --n-max 10 --check-unused 10 --max-age 300
