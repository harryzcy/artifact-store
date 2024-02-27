FROM rust:1.76.0-bookworm@sha256:a71cd88f9dd32fbdfa67c935f55165ddd89b7166e95de6c053c9bf33dd7381d5 AS builder

WORKDIR /app

COPY ./ .

RUN apt-get update && apt-get install -y libclang-dev
RUN cargo build --release
RUN mkdir /data

FROM busybox:1.36.1-glibc@sha256:e046063223f7eaafbfbc026aa3954d9a31b9f1053ba5db04a4f1fdc97abd8963 as tools

FROM gcr.io/distroless/cc-debian12@sha256:db482c5c6ef39662a0b55a718e19b8c0cabfbbd57c2dd342f1791c4f096d0b7b

WORKDIR /app

COPY --from=builder --chown=nonroot:nonroot /data /data
COPY --from=builder /app/target/release/artifact-store /bin
COPY --from=tools /bin/wget /bin/

USER nonroot:nonroot
EXPOSE 3001

HEALTHCHECK --interval=60s --timeout=30s --start-period=5s --retries=3 \
  CMD [ "wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:3001/ping", "||", "exit", "1" ]

CMD [ "/bin/artifact-store" ]
