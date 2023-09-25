FROM rust:latest

RUN apt-get update && apt-get upgrade -y
RUN apt-get install -y g++-mingw-w64-x86-64

RUN rustup default nightly

RUN rustup target add x86_64-pc-windows-gnu
RUN rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
RUN rustup toolchain install nightly-x86_64-pc-windows-gnu

WORKDIR /app