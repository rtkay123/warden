FROM rust:1.91.1-slim AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev protobuf-compiler curl
RUN update-ca-certificates

WORKDIR /usr/src/app

RUN mkdir -p crates

COPY ./crates/router crates/router
COPY ./lib lib
COPY ./Cargo.toml .
COPY ./Cargo.lock .

RUN cargo fetch

COPY ./proto proto

RUN cargo build --target x86_64-unknown-linux-musl --release

FROM scratch
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/warden-router ./
CMD [ "./warden-router" ]
