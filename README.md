# Valve

The purpose of `valve` is to create a multi-threaded plumber API utilizing `libR-sys` and the plumber R package.

Plumber is an R package that creates RESTful APIs from R functions. It is limited in that each API is a single R process and thus a single thread. The intention of this Rust crate is to be able to spawn multiple threads each with their own Plumber API leveraging Rust's "fearless concurrency."

## Design Idea

- a plumber API is defined by `plumber.R`
- `n` R processes are defined
    - each process spawns a plumber API at a different port
- Rust handles requests and directs them to an available plumber API
- Returns the curl response from the plumber API directly

-----
