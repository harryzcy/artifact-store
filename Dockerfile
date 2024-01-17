FROM rust:1.75.0-bookworm@sha256:755b46a632bc0c1b95aa8a8babfca92c3f8f91e041d3095132b5f5c8ec99bd22 AS builder

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
