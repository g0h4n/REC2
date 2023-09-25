FROM rust:1.72-slim-buster

WORKDIR /usr/src/rec2

RUN \
	apt-get -y update && \
	apt-get -y install gcc clang libclang-dev musl-tools make gcc-mingw-w64-x86-64 pkg-config libudev-dev libssl-dev && \
	rm -rf /var/lib/apt/lists/*

ENTRYPOINT ["make"]