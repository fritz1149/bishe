FROM rust:alpine3.17 as builder
WORKDIR /work/
COPY . .
RUN cargo build --bin dispatcher

FROM alpine:3.17
EXPOSE 3000
COPY --from=builder /work/target/debug/dispatcher /work/dispatcher
WORKDIR /work/
CMD ["./dispatcher"]