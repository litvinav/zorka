FROM rust:1.67-alpine3.17
WORKDIR /app

COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml

RUN apk add build-base libressl-dev --no-cache \
 && cargo build --release

ENV DATABASE_URL="./sqlite.db"
ENV RUST_LOG="zorka=error"

ENTRYPOINT ["/app/target/release/zorka"]
