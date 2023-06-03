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

COPY --chown=zorka:zorka ./assets             ./assets
COPY --chown=zorka:zorka ./templates          ./templates
COPY --chown=zorka:zorka ./configuration.yaml ./configuration.yaml
COPY --chown=zorka:zorka --from=builder       /app/target/release/zorka /app/zorka

RUN apk add curl --no-cache && rm /usr/sbin/sendmail \
 && addgroup zorka && adduser -S zorka -G zorka -H -D \
 && mkdir -p /app/backups \
 && chown -hR zorka:zorka /app

USER zorka
EXPOSE 8080
ENTRYPOINT ["/app/zorka"]
