FROM rust:1.77.2-bookworm@sha256:371ae5168b632596c4b32fb97f63bcf85c26cb84cf5e775da4e34e42c1651626 AS builder

WORKDIR /app

COPY ./ .

RUN apt-get update && apt-get install -y libclang-dev
RUN CARGO_INCREMENTAL=0 cargo build --release
RUN mkdir /data

FROM busybox:1.36.1-glibc@sha256:735f3e2573d5540476f997fbf562907d6901f995d6feb0e18132fade186f2230 as tools

FROM gcr.io/distroless/cc-debian12@sha256:eed8bd290a9f83d0451e7812854da87a8407f1d68f44fae5261c16556be6465b

WORKDIR /app

COPY --from=builder --chown=nonroot:nonroot /data /data
COPY --from=builder /app/target/release/artifact-store /bin
COPY --from=tools /bin/wget /bin/

USER nonroot:nonroot
EXPOSE 3001

HEALTHCHECK --interval=60s --timeout=30s --start-period=5s --retries=3 \
  CMD [ "wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:3001/ping", "||", "exit", "1" ]

CMD [ "/bin/artifact-store" ]
