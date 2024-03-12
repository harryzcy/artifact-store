FROM rust:1.76.0-bookworm@sha256:969ca542302e38158733fdcb9ff465541391450691817ec011bb2fdffc3f64a8 AS builder

WORKDIR /app

COPY ./ .

RUN apt-get update && apt-get install -y libclang-dev
RUN cargo build --release
RUN mkdir /data

FROM busybox:1.36.1-glibc@sha256:8425131865cec8fba4d2db137c883902155e0d58fcbb301690693161cc903910 as tools

FROM gcr.io/distroless/cc-debian12@sha256:e6ae66a5a343d7112167f9117c4e630cfffcd80db44e44302759ec13ddd2d22b

WORKDIR /app

COPY --from=builder --chown=nonroot:nonroot /data /data
COPY --from=builder /app/target/release/artifact-store /bin
COPY --from=tools /bin/wget /bin/

USER nonroot:nonroot
EXPOSE 3001

HEALTHCHECK --interval=60s --timeout=30s --start-period=5s --retries=3 \
  CMD [ "wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:3001/ping", "||", "exit", "1" ]

CMD [ "/bin/artifact-store" ]
