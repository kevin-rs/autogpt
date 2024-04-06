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

FROM alpine:3.19
RUN addgroup -S kevin && \
    adduser -S -G kevin kevin

RUN apk add --no-cache sudo

RUN addgroup -S sudo

RUN addgroup kevin sudo

RUN echo "%sudo ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

WORKDIR /home/kevin

COPY --from=builder /kevin/target/release/autogpt /usr/local/bin/autogpt

USER kevin

ENTRYPOINT [ "/usr/local/bin/autogpt" ]
