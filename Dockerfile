FROM rust:1.85.0@sha256:e91bad14749caddcb9f203d2b95096d80a3a4e57f1b2097187f308b36011b871 AS builder

WORKDIR /app

COPY ./ .

RUN apt-get update && apt-get install -y libclang-dev
RUN CARGO_INCREMENTAL=0 cargo build --release
RUN mkdir /data

FROM busybox:1.37.0@sha256:498a000f370d8c37927118ed80afe8adc38d1edcbfc071627d17b25c88efcab0 as tools

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
