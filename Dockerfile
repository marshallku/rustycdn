FROM rust:1.73-alpine AS base

WORKDIR /usr/src/rustycdn

RUN set -eux; \
    apk add --no-cache musl-dev pkgconfig libressl-dev; \
    rm -rf $CARGO_HOME/registry

COPY Cargo.* .

RUN mkdir src && \
    echo 'fn main() {println!("Hello, world!");}' > src/main.rs && \
    cargo build --release && \
    rm target/release/rustycdn* && \
    rm target/release/deps/rustycdn* && \
    rm -rf src

FROM base AS builder

COPY src src
RUN cargo build --release

FROM alpine:3.14

WORKDIR /usr/local/bin

COPY --from=builder /usr/src/rustycdn/target/release/rustycdn .

EXPOSE 41890

CMD ["./rustycdn"]