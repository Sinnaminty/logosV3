########## Chef it up! ##########
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
# Bindgen / libclang for build.rs
RUN apt-get update && apt-get install -y \
    clang llvm-dev libclang-dev pkg-config build-essential ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json

# build.rs needs DECtalk headers + libs for bindgen and linking
COPY vendor/ vendor/

# Cache build dependencies
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release

########## Runtime ##########
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

RUN useradd -m app
WORKDIR /app

# Copy the compiled binary
COPY --from=builder /app/target/release/logosV3 /app/bot

# DECtalk runtime: shared objects, dictionaries, and config
COPY vendor/dectalk/dist/*.so vendor/dectalk/dist/
COPY vendor/dectalk/dist/*.dic vendor/dectalk/dist/
COPY DECtalk.conf .

ENV LD_LIBRARY_PATH=/app/vendor/dectalk/dist
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1
USER app

CMD ["/app/bot"]
