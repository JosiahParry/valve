FROM rust:1.71.1-alpine AS builder
RUN apk add musl-dev
RUN cargo install valve-rs --no-default-features

FROM rhub/r-minimal:4.3.1

# Copy the binary from the builder stage
COPY --from=builder /usr/local/cargo/bin/valve /usr/local/bin/

# Install additional packages and perform other setup steps
RUN apk add --no-cache --update-cache \
        --repository http://nl.alpinelinux.org/alpine/v3.11/main \
        autoconf=2.69-r2 \
        automake=1.16.1-r0 && \
    installr -d \
        -t "libsodium-dev curl-dev linux-headers autoconf automake" \
        -a libsodium \
        plumber

EXPOSE 8000
COPY plumber.R /api/plumber.R

# Start Valve app
ENTRYPOINT valve -f /api/plumber.R -h 0.0.0.0 -p 8000 --workers 3 --n-max 3 --check-unused 10 --max-age 300

