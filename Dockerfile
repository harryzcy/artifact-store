FROM rust:1.71.0-bullseye AS builder

WORKDIR /app

COPY ./ .

RUN cargo build --release

FROM gcr.io/distroless/cc-debian11

WORKDIR /app

COPY --from=builder /app/target/release/artifact-store ./

USER nonroot:nonroot

CMD ["/app/artifact-store"]
