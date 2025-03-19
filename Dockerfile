FROM rust:1.85.1@sha256:e51d0265072d2d9d5d320f6a44dde6b9ef13653b035098febd68cce8fa7c0bc4 AS builder

WORKDIR /app

COPY ./ .

RUN apt-get update && apt-get install -y libclang-dev
RUN CARGO_INCREMENTAL=0 cargo build --release
RUN mkdir /data

FROM busybox:1.37.0@sha256:37f7b378a29ceb4c551b1b5582e27747b855bbfaa73fa11914fe0df028dc581f as tools

FROM gcr.io/distroless/cc-debian12@sha256:85dac24dd2f03e841d986d5ed967385d3a721dcd9dbd21b602ddd82437f364c9

WORKDIR /app

COPY --from=builder --chown=nonroot:nonroot /data /data
COPY --from=builder /app/target/release/artifact-store /bin
COPY --from=tools /bin/wget /bin/

USER nonroot:nonroot
EXPOSE 3001

HEALTHCHECK --interval=60s --timeout=30s --start-period=5s --retries=3 \
  CMD [ "wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:3001/ping", "||", "exit", "1" ]

CMD [ "/bin/artifact-store" ]
