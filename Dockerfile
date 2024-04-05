FROM rust:alpine as builder
LABEL maintainer="kevin-rs <https://github.com/kevin-rs>"
LABEL author="Mahmoud Harmouch <https://github.com/wiseaidev>"
RUN apk update && apk upgrade
RUN apk add --no-cache build-base pkgconfig libressl-dev

WORKDIR /kevin
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY README.md ./
RUN cargo build --release --all-features

FROM alpine:3.17
RUN addgroup -S kevin && \
    adduser -S -G kevin kevin

USER kevin
COPY --from=builder /kevin/target/release/autogpt /usr/local/bin/autogpt
ENTRYPOINT [ "/usr/local/bin/autogpt" ]