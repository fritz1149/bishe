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
EXPOSE 3000
COPY --from=builder /app/target/release/monitor /work/monitor
WORKDIR work
ENTRYPOINT ["/usr/bin/env"]
CMD ["./monitor"]