FROM rust:1.91.0-slim AS builder

ARG FEATURES=""

ENV SQLX_OFFLINE=true

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev protobuf-compiler curl
RUN update-ca-certificates

WORKDIR /usr/src/app

RUN mkdir -p crates

COPY ./.sqlx .sqlx
COPY ./crates/warden crates/warden
COPY ./lib lib
COPY ./Cargo.toml .
COPY ./Cargo.lock .

RUN cargo fetch

COPY ./proto proto

RUN cargo build -F "${FEATURES}" --target x86_64-unknown-linux-musl --release

FROM scratch
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/warden ./
CMD [ "./warden" ]
