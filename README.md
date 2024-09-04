# Blog CDN

[![CI](https://github.com/marshallku/marshallku-blog-cdn/actions/workflows/ci.yml/badge.svg)](https://github.com/marshallku/marshallku-blog-cdn/actions/workflows/ci.yml)
[![Deploy to Container Registry](https://github.com/marshallku/marshallku-blog-cdn/actions/workflows/deploy.yml/badge.svg)](https://github.com/marshallku/marshallku-blog-cdn/actions/workflows/deploy.yml)

![Quality Gate Status](https://badge.marshallku.dev?metric=alert_status&project=marshallku_marshallku-blog-cdn_7201a95a-ba17-439f-ac2d-60f1c9624f4c)
![Bugs](https://badge.marshallku.dev?metric=bugs&project=marshallku_marshallku-blog-cdn_7201a95a-ba17-439f-ac2d-60f1c9624f4c)
![Code Smells](https://badge.marshallku.dev?metric=code_smells&project=marshallku_marshallku-blog-cdn_7201a95a-ba17-439f-ac2d-60f1c9624f4c)
![Maintainability Rating](https://badge.marshallku.dev?metric=sqale_rating&project=marshallku_marshallku-blog-cdn_7201a95a-ba17-439f-ac2d-60f1c9624f4c)
![Reliability Rating](https://badge.marshallku.dev?metric=reliability_rating&project=marshallku_marshallku-blog-cdn_7201a95a-ba17-439f-ac2d-60f1c9624f4c)
![Security Rating](https://badge.marshallku.dev?metric=security_rating&project=marshallku_marshallku-blog-cdn_7201a95a-ba17-439f-ac2d-60f1c9624f4c)
![Vulnerabilities](https://badge.marshallku.dev?metric=vulnerabilities&project=marshallku_marshallku-blog-cdn_7201a95a-ba17-439f-ac2d-60f1c9624f4c)
![Coverage](https://badge.marshallku.dev?metric=coverage&project=marshallku_marshallku-blog-cdn_7201a95a-ba17-439f-ac2d-60f1c9624f4c)

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

## Removing files with cronjob

Automate the cleanup of unused files in your CDN's root directory by setting up cron jobs. These commands will search for and delete files based on their age and type, ensuring your directory remains clutter-free.

### Setup

Paste the following lines into your crontab file to schedule the cleanup tasks. These commands are executed daily at 4:00 AM.

- The first job removes CSS and JavaScript files that haven't been accessed in over 5 days

    ```shell
    00 4 * * * /usr/bin/find /home/ubuntu/cdn/cdn_root -mindepth 2 -atime +5 -type f \( -o -iname \*.css -o -iname \*.js \) | xargs rm 1>/dev/null 2>/dev/null
    ```

- The second job targets image and video files (PNG, JPG, JPEG, GIF, WEBP, MP4, WEBM, SVG) as well as CSS and JavaScript files that haven't been accessed in over a year (365 days):

    ```shell
    00 4 * * * /usr/bin/find /home/ubuntu/cdn/cdn_root -mindepth 2 -atime +365 -type f \( -iname \*.png -o -iname \*.jpg -o -iname \*.jpeg -o -iname \*.gif -o -iname \*.webp -o -iname \*.mp4 -o -iname \*.webm -o -iname \*.svg -o -iname \*.css -o -iname \*.js \) | xargs rm 1>/dev/null 2>/dev/null
    ```
