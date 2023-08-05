FROM rust:1.71.1-bullseye AS builder

WORKDIR /app

COPY ./ .

RUN apt-get update && apt-get install -y libclang-dev
RUN cargo build --release

FROM gcr.io/distroless/cc-debian11

WORKDIR /app

COPY --from=builder /app/target/release/artifact-store ./

RUN mkdir /data && chown -R nonroot:nonroot /data && chown nonroot:nonroot /data

USER nonroot:nonroot
EXPOSE 3001

CMD ["/app/artifact-store"]
