FROM rust:1.76.0-bookworm@sha256:a71cd88f9dd32fbdfa67c935f55165ddd89b7166e95de6c053c9bf33dd7381d5 AS builder

WORKDIR /app

COPY ./ .

RUN apt-get update && apt-get install -y libclang-dev
RUN cargo build --release
RUN mkdir /data

FROM gcr.io/distroless/cc-debian12@sha256:efafe74d452c57025616c816b058e3d453c184e4b337897a8d38fef5026b079d

WORKDIR /app

COPY --from=builder --chown=nonroot:nonroot /data /data
COPY --from=builder /app/target/release/artifact-store ./

USER nonroot:nonroot
EXPOSE 3001

HEALTHCHECK --interval=60s --timeout=30s --start-period=5s --retries=3 \
  CMD [ "wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:3001/ping", "||", "exit", "1" ]

CMD [ "/app/artifact-store" ]
