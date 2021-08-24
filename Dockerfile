FROM rust:latest AS build
WORKDIR /build
COPY . .
RUN cargo build --release

FROM debian:stable-slim
RUN apt-get update && apt-get install tini && apt-get clean
WORKDIR /app
COPY --from=build /build/target/release/bin .
COPY --from=build /build/config/ config
VOLUME [ "/app/logs" ]
ENTRYPOINT ["/usr/bin/tini", "--", "/app/bin"]
