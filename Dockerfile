FROM rust:latest AS build
WORKDIR /build
COPY . .
RUN cargo build --release

FROM debian:stable-slim
RUN apt-get update && apt-get install -y tini openssl && apt-get clean
WORKDIR /app
COPY --from=build /build/target/release/KraBot .
ENTRYPOINT ["/usr/bin/tini", "--", "/app/KraBot"]
