FROM rust:1.73-alpine AS builder

WORKDIR /usr/src/marshallku-blog-cdn

RUN set -eux; \
    apk add --no-cache musl-dev pkgconfig libressl-dev; \
    rm -rf $CARGO_HOME/registry

COPY . .
RUN cargo build --release

FROM alpine:3.14

WORKDIR /usr/local/bin

COPY --from=builder /usr/src/marshallku-blog-cdn/target/release/marshallku-blog-cdn .

EXPOSE 41890

CMD ["./marshallku-blog-cdn"]