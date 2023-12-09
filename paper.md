---
title: "Valve: scaling R for production"
subtitle: "Redirecting your plumbing for you"
author:
  - name: Josiah Parry
    orcid: "0000-0001-9910-865X"
    affiliation: Environmental Systems Research Institute, Redlands, CA, USA
tags:
  - R
  - Rust
bibliograph: paper.bib
---

# Summary

Valve is a Rust command line interface (CLI) tool with an accompanying R package designed to scale R in production environments. At its core, Valve is a web server that runs REST APIs created with the R package plumber in parallel. Valve handles connection pooling, auto-scaling, and thread termination making it an exceptionally powerful way to serve R based web-services into a production environment. The accompanying R package makes it even easier for R users to auto scale existing plumber APIs without knowledge of the command line. The intention of valve is to be able to make scaling plumber APIs, and thus R itself, easier.

# Statement of need

In the R ecosystem there are a number of ways to create REST APIs. These include RestRserve [@restr], FastRWeb [@fastrw], Ambiorix [@ambio], httpuv [@httpuv], and plumber [@plumber]. RestRserve and FastRWeb are built upon Rserve [@rserve] which is a TCP/IP server written in Java with a rich history dating back to 2006. While RestRserve and FastRWeb support multi-threading and are highly performant in their own right, the dependency on Rserve make it very challenging to put into production. On the other hand, the package plumber, first developed by RStudio, now Posit, is a higher level alternative that is built upon httpuv (as is ambiorix). While plumber does not boast the same level of performance that can be achieved using RestRserve, its user friendly design has made it the de facto standard for building web services in R.

httpuv, and by extension plumber, run on a single active R connection. That means each request that comes in has to be added to a queue. The next request cannot be processed until the previous one has been. Under high load, a single connection cannot meet demand. In a production setting, APIs are often deployed in a Docker container. Containers often have multiple threads available to them that go unused when serving a plumber API. Valve ensures that the maximum amount of performance can be eked out when serving a plumber API. 

# Design and implementation

Valve is built specifically for plumber, using Rust [@rust] and leveraging the power the axum [@axum] web framework and Deadpool [@deadpool] for asynchronous connection pooling. The architecture of valve is captured roughly in the below image.


There are three primary components to a Valve application: the web-server built using axum, the asynchronous tokio runtime, and the Deadpool connection pool.

Instead of a single single plumber API handling all incoming requests we define a main web-server on a single port. That web-server's runtime has a predefined number of worker that are responsible for all managing incoming traffic. The inbound requests are send to the main port which are passed to the axum **router**. The router then requests an available plumber API from the connection pool. When the connection is received, the request is sent to the plumber API and the response is captured by the worker, and delivered as the response. In essence, the main web-server acts as a reverse proxy for the plumber APIs in the connection pool.

## Auto-scaling 


# Running a valve app


## Docker compatibility

------ 

## References 

https://ambiorix.dev/
https://rforge.net/Rserve/
https://rforge.net/FastRWeb/
