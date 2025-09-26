FROM rust:1.90.0@sha256:eabb786e74b520e7ea45baca03ea20c3e8c6dc037c392d457badf05d8e5818b5 AS builder

WORKDIR /app

COPY ./ .

RUN apt-get update && apt-get install --no-install-recommends -y libclang-dev
RUN CARGO_INCREMENTAL=0 cargo build --release
RUN mkdir /data

FROM busybox:1.37.0@sha256:d82f458899c9696cb26a7c02d5568f81c8c8223f8661bb2a7988b269c8b9051e AS tools

FROM gcr.io/distroless/cc-debian12@sha256:620d8b11ae800f0dbd7995f89ddc5344ad603269ea98770588b1b07a4a0a6872

WORKDIR /app

COPY --from=builder --chown=nonroot:nonroot /data /data
COPY --from=builder /app/target/release/artifact-store /bin
COPY --from=tools /bin/wget /bin/

USER nonroot:nonroot
EXPOSE 3001

HEALTHCHECK --interval=60s --timeout=30s --start-period=5s --retries=3 \
  CMD [ "wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:3001/ping", "||", "exit", "1" ]

CMD [ "/bin/artifact-store" ]
