FROM rust:1.83.0-slim as builder

WORKDIR /usr/src/

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /usr/app

COPY --from=builder /usr/src/assets assets
COPY --from=builder /usr/src/config config
COPY --from=builder /usr/src/target/release/gitcrab-cli gitcrab-cli

EXPOSE 5150

CMD ["/usr/app/gitcrab-cli", "start"]
