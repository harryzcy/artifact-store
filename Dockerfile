FROM rust:1.75.0-bookworm@sha256:e1730890f6646c5d896cc6867c064fc986b6160f362f3cac7fe425c94ccbd8f2 AS builder

WORKDIR /app

COPY ./ .

RUN apt-get update && apt-get install -y libclang-dev
RUN cargo build --release
RUN mkdir /data

FROM gcr.io/distroless/cc-debian12@sha256:4049e8f163161818a52e028c3c110ee0ba9d71a14760ad2838aabba52b3f9782

WORKDIR /app

COPY --from=builder --chown=nonroot:nonroot /data /data
COPY --from=builder /app/target/release/artifact-store ./

USER nonroot:nonroot
EXPOSE 3001

CMD ["/app/artifact-store"]
