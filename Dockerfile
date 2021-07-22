FROM rust:latest AS build
WORKDIR /build
COPY . .
RUN cargo build --release

FROM rust:slim
WORKDIR /app
COPY --from=build /build/target/release/bin .
COPY --from=build /build/config/ config
VOLUME [ "/app/logs" ]
ENTRYPOINT ["/app/bin"]
