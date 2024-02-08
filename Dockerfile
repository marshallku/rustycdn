FROM rust:1.73-alpine AS chef

WORKDIR /usr/src/marshallku-blog-cdn

RUN set -eux; \
    apk add --no-cache musl-dev pkgconfig libressl-dev; \
    cargo install cargo-chef; \
    rm -rf $CARGO_HOME/registry

FROM chef as planner

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /usr/src/marshallku-blog-cdn/recipe.json .
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release

FROM alpine:3.14

WORKDIR /usr/local/bin

COPY --from=builder /usr/src/marshallku-blog-cdn/target/release/marshallku-blog-cdn .

EXPOSE 41880

CMD ["./marshallku-blog-cdn"]