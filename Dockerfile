FROM rust:1.68-alpine3.17
WORKDIR /app

COPY ./configuration.yaml ./Cargo.* ./
COPY ./src ./src
COPY ./assets ./assets
COPY ./templates ./templates

RUN apk --update add build-base libressl-dev \
 && cargo build --release \
 && rm -rf src

ENV DATABASE_URL="./sqlite.db"
ENV RUST_LOG="zorka=error"

ENTRYPOINT ["/app/target/release/zorka"]
