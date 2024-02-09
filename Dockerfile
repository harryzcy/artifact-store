FROM rust:1.76.0-bookworm@sha256:8e876026c9dbf43a12c2a15476a5d8f81cc41a56acbb975f04f859170fd0a105 AS builder

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
