FROM rust:1.76.0-bookworm@sha256:e21cbd18f8bafcf552b78ef62b4b2ccd5cbbc9df739d26ee1836d46fb2591d2f AS builder

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
