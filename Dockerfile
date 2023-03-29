# - - - # - - - 🏗️ - - - # - - - #
FROM rust:1.68-alpine3.17 as builder
WORKDIR /app/

COPY ./Cargo.*  ./
COPY ./src      ./src

RUN apk --update add build-base \
 && cargo build --release

# - - - # - - - 🐋 - - - # - - - #
FROM alpine:3.17 as runtime
WORKDIR /app/

ENV RUST_LOG="zorka=error"

COPY ./assets             ./assets
COPY ./templates          ./templates
COPY ./configuration.yaml ./configuration.yaml
COPY --from=builder       /app/target/release/zorka /app/zorka

RUN apk add curl --no-cache
EXPOSE 8080
ENTRYPOINT ["/app/zorka"]
