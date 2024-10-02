FROM rust:1.81.0@sha256:a21d54019c66e3a1e7512651e9a7de99b08f28d49b023ed7220b7fe4d3b9f24e AS builder

WORKDIR /app

COPY ./ .

RUN apt-get update && apt-get install -y libclang-dev
RUN CARGO_INCREMENTAL=0 cargo build --release
RUN mkdir /data

FROM busybox:1.36.1-glibc@sha256:ebc6796a3eeca8489685eeb1f35223bc2268eed81c23c1c11c2a5713dfaa9910 as tools

FROM gcr.io/distroless/cc-debian12@sha256:3310655aac0d85eb9d579792387af1ff3eb7a1667823478be58020ab0e0d97a8

WORKDIR /app

COPY --from=builder --chown=nonroot:nonroot /data /data
COPY --from=builder /app/target/release/artifact-store /bin
COPY --from=tools /bin/wget /bin/

USER nonroot:nonroot
EXPOSE 3001

HEALTHCHECK --interval=60s --timeout=30s --start-period=5s --retries=3 \
  CMD [ "wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:3001/ping", "||", "exit", "1" ]

CMD [ "/bin/artifact-store" ]
