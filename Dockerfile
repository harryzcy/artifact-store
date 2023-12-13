FROM rust:1.74.1-bookworm@sha256:32d220ca8c77fe56afd6d057c382ea39aced503278526a34fc62b90946f92e02 AS builder

WORKDIR /app

COPY ./ .

RUN apt-get update && apt-get install -y libclang-dev
RUN cargo build --release
RUN mkdir /data

FROM gcr.io/distroless/cc-debian12@sha256:4ddfea445cfeed54d6c9a1e51b97e7f3a5087f3a6a69cb430ebba3a89c402a41

WORKDIR /app

COPY --from=builder --chown=nonroot:nonroot /data /data
COPY --from=builder /app/target/release/artifact-store ./

USER nonroot:nonroot
EXPOSE 3001

CMD ["/app/artifact-store"]
