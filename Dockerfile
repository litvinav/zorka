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

COPY --chown=zorka:zorka ./assets             ./assets
COPY --chown=zorka:zorka ./templates          ./templates
COPY --chown=zorka:zorka ./configuration.yaml ./configuration.yaml
COPY --chown=zorka:zorka --from=builder       /app/target/release/zorka /app/zorka

EXPOSE 8080
RUN apk add curl --no-cache && rm /usr/sbin/sendmail \
 && addgroup zorka && adduser -S zorka -G zorka -H -D

USER zorka
ENTRYPOINT ["/app/zorka"]
