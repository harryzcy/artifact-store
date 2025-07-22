FROM rust:1.88.0@sha256:a5c5c4b34e4742c068ed35e31c171443d4be3a417e419d03339dad7001d3f903 AS builder

WORKDIR /app

COPY ./ .

RUN apt-get update && apt-get install -y libclang-dev
RUN CARGO_INCREMENTAL=0 cargo build --release
RUN mkdir /data

FROM busybox:1.37.0@sha256:f85340bf132ae937d2c2a763b8335c9bab35d6e8293f70f606b9c6178d84f42b AS tools

FROM gcr.io/distroless/cc-debian12@sha256:eccec5274132c1be0ce5d2c8e6fe41033e64af5e987ccee9007826e4c012069d

WORKDIR /app

COPY --from=builder --chown=nonroot:nonroot /data /data
COPY --from=builder /app/target/release/artifact-store /bin
COPY --from=tools /bin/wget /bin/

USER nonroot:nonroot
EXPOSE 3001

HEALTHCHECK --interval=60s --timeout=30s --start-period=5s --retries=3 \
  CMD [ "wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:3001/ping", "||", "exit", "1" ]

CMD [ "/bin/artifact-store" ]
