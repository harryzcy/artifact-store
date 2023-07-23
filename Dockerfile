FROM rust:latest AS builder

WORKDIR /app

COPY ./ .

RUN cargo build --release

FROM gcr.io/distroless/cc-debian11:latest

WORKDIR /app

COPY --from=builder /app/target/release/artifact-store ./

USER nonroot:nonroot

CMD ["/app/artifact-store"]
