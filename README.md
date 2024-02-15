# Blog CDN

This project is a custom CDN(Content Delivery Network) server built with Rust, designed to efficiently serve static files and images for my personal blog.

## Features

- Static File Serving: Serves CSS, JS, and other static files required by the blog.
- Dynamic Image Resizing: On-demand image resizing to serve optimized images based on request parameters.
- Caching Mechanism: Fetches static files from a specified origin and caches them locally to speed up subsequent requests.

## Prerequisites

- Rust
- Docker
- Basic understanding of Nginx

### Additional packages

```bash
sudo apt install pkg-config libssl-dev
```

In order to run the application using `cargo run`, the `reqwest` library requires the `pkg-config` and `libssl-dev` packages to be installed

## Usage

After starting the server, it will listen for requests on the configured address. Static files and images can be accessed through the`/files/*path` and `/images/*path` endpoints, respectively.

## Configuration

- `BIND_ADDRESS`: Sets the IP address the server listens on (default: `127.0.0.1`).
- `PORT`: Sets the port the server listens on (default: `41890`).
- `HOST`: Sets the host server that original files exist (default: `http://localhost/`).

## Production Deployment with Nginx

For optimal performance and reliability in a production environment, it is recommended to deploy the Rust server behind Nginx. This setup enhances security, load balancing, and static asset serving capabilities. An example Nginx configuration tailored for use with this server can be found in `config/nginx.conf`. Refer to this example to configure Nginx as a reverse proxy for your Rust CDN server.
