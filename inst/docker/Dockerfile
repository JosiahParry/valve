FROM rocker/r-ver:4.3.0
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
EXPOSE 8000

# Install Rust toolchain & add to the path
RUN apt-get install -y -q \
    build-essential \
    curl  
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Valve binary from Github
RUN cargo install valve-rs --no-default-features

# Start Valve app
ENTRYPOINT valve -f /api/plumber.R -h 0.0.0.0 -p 8000 --workers 10 --n-max 5 --check-unused 10 --max-age 999