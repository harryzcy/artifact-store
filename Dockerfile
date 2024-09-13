FROM rust:1.81.0@sha256:fcd390e0a3a6bfcf26969861efbe7b864df052aa71a361cf3cd7c5c585b1b413 AS builder

WORKDIR /app

COPY ./ .

RUN apt-get update && apt-get install -y libclang-dev
RUN CARGO_INCREMENTAL=0 cargo build --release
RUN mkdir /data

FROM busybox:1.36.1-glibc@sha256:949757861bcee7514f64d9b44d3c1d43c21f5183cae113e97b98261fc1c522dc as tools

FROM gcr.io/distroless/cc-debian12@sha256:b6e1e913f633495eeb80a41e03de1a41aa863e9b19902309b180ffdc4b99db2c

WORKDIR /app

COPY --from=builder --chown=nonroot:nonroot /data /data
COPY --from=builder /app/target/release/artifact-store /bin
COPY --from=tools /bin/wget /bin/

USER nonroot:nonroot
EXPOSE 3001

HEALTHCHECK --interval=60s --timeout=30s --start-period=5s --retries=3 \
  CMD [ "wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:3001/ping", "||", "exit", "1" ]

CMD [ "/bin/artifact-store" ]
