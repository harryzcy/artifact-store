FROM rust:1.94.0@sha256:335533f479eeb7796d65c4a9e09424e5b5f7de75b96cde6fcbbf267c69497de1 AS builder

WORKDIR /app

COPY ./ .

RUN apt-get update && apt-get install --no-install-recommends -y libclang-dev
RUN CARGO_INCREMENTAL=0 cargo build --release
RUN mkdir /data

FROM busybox:1.37.0@sha256:b3255e7dfbcd10cb367af0d409747d511aeb66dfac98cf30e97e87e4207dd76f AS tools

FROM gcr.io/distroless/cc-debian13@sha256:8c1a496b055d36c222e95ebd5c53bdd1fee447689a97ad889febca8380d578ec

WORKDIR /app

COPY --from=builder --chown=nonroot:nonroot /data /data
COPY --from=builder /app/target/release/artifact-store /bin
COPY --from=tools /bin/wget /bin/

USER nonroot:nonroot
EXPOSE 3001

HEALTHCHECK --interval=60s --timeout=30s --start-period=5s --retries=3 \
  CMD [ "wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:3001/ping", "||", "exit", "1" ]

CMD [ "/bin/artifact-store" ]
