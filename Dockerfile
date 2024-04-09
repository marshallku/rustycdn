FROM rust:1.73-alpine AS base

WORKDIR /usr/src/marshallku-blog-cdn

RUN set -eux; \
    apk add --no-cache musl-dev pkgconfig libressl-dev; \
    rm -rf $CARGO_HOME/registry

COPY Cargo.* .

RUN mkdir src && \
    echo 'fn main() {println!("Hello, world!");}' > src/main.rs && \
    cargo build --release && \
    rm target/release/marshallku-blog-cdn* && \
    rm target/release/deps/marshallku_blog_cdn* && \
    rm -rf src

FROM base AS builder

COPY src src
RUN cargo build --release

FROM alpine:3.14

WORKDIR /usr/local/bin

COPY --from=builder /usr/src/marshallku-blog-cdn/target/release/marshallku-blog-cdn .

EXPOSE 41890

CMD ["./marshallku-blog-cdn"]