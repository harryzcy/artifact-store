FROM rust:1.77.2-bookworm@sha256:6052afe7c422c163798bb9064b7215db15c5f790214ee2c2e787daf8ed3de92a AS builder

WORKDIR /app

COPY ./ .

RUN apt-get update && apt-get install -y libclang-dev
RUN CARGO_INCREMENTAL=0 cargo build --release
RUN mkdir /data

FROM busybox:1.36.1-glibc@sha256:cb6aeb580841ccd038a2fb39a9d89948a4eced95ed02c1f726d599da65c8f0c5 as tools

FROM gcr.io/distroless/cc-debian12@sha256:7a01d633f75120af59c71489e0911fa8b6512673a3ff0b999522b4221ab4d86a

WORKDIR /app

COPY --from=builder --chown=nonroot:nonroot /data /data
COPY --from=builder /app/target/release/artifact-store /bin
COPY --from=tools /bin/wget /bin/

USER nonroot:nonroot
EXPOSE 3001

HEALTHCHECK --interval=60s --timeout=30s --start-period=5s --retries=3 \
  CMD [ "wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:3001/ping", "||", "exit", "1" ]

CMD [ "/bin/artifact-store" ]
