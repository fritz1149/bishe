FROM 19231149/rust-builder AS chef
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin monitor

# We do not need the Rust toolchain to run the binary!
FROM networkstatic/iperf3:latest AS runtime
RUN apt-get update && apt-get install -y inetutils-ping
EXPOSE 5201
COPY --from=builder /app/target/release/monitor /app/monitor
COPY --from=builder /app/run.sh /app/run.sh
WORKDIR /app
RUN chmod +x ./run.sh
ENTRYPOINT ["./run.sh"]
#ENTRYPOINT ["./monitor"]