FROM rust:1.75.0-bookworm@sha256:1c5eaf3e66465de5cdd71f66b0ada48cec56d9c267f67880785fb6a29085c444 AS builder

WORKDIR /app

COPY ./ .

RUN apt-get update && apt-get install -y libclang-dev
RUN cargo build --release
RUN mkdir /data

FROM gcr.io/distroless/cc-debian12@sha256:6714977f9f02632c31377650c15d89a7efaebf43bab0f37c712c30fc01edb973

WORKDIR /app

COPY --from=builder --chown=nonroot:nonroot /data /data
COPY --from=builder /app/target/release/artifact-store ./

USER nonroot:nonroot
EXPOSE 3001

CMD ["/app/artifact-store"]
