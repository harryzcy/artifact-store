FROM rust:1.91.1@sha256:cd34b27bc6df5450e4952075dc6bd3881a1aeb8f5d0478cd75c2160ca47e2182 AS builder

WORKDIR /app

COPY ./ .

RUN apt-get update && apt-get install --no-install-recommends -y libclang-dev
RUN CARGO_INCREMENTAL=0 cargo build --release
RUN mkdir /data

FROM busybox:1.37.0@sha256:e3652a00a2fabd16ce889f0aa32c38eec347b997e73bd09e69c962ec7f8732ee AS tools

FROM gcr.io/distroless/cc-debian13@sha256:68db2bf2b975ff277c9b2b569c327e47e2824e2c143f4dfe7c4027b15ff2f931

WORKDIR /app

COPY --from=builder --chown=nonroot:nonroot /data /data
COPY --from=builder /app/target/release/artifact-store /bin
COPY --from=tools /bin/wget /bin/

USER nonroot:nonroot
EXPOSE 3001

HEALTHCHECK --interval=60s --timeout=30s --start-period=5s --retries=3 \
  CMD [ "wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:3001/ping", "||", "exit", "1" ]

CMD [ "/bin/artifact-store" ]
