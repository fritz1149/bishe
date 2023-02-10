FROM rust:latest as builder
WORKDIR /work/
COPY . .
RUN cargo build --bin dispatcher --release

FROM busybox:glibc
EXPOSE 3000
COPY --from=builder /work/target/release/dispatcher /work/dispatcher
WORKDIR /work/
CMD ["./dispatcher"]
