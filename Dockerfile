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

# cache build dependencies
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
# rebuild real binary
RUN cargo build --release

########## Runtime ##########
FROM debian:bookworm-slim

# Minimal runtime deps
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates pulseaudio && \
    rm -rf /var/lib/apt/lists/*

# Run as non-root
RUN useradd -m app
WORKDIR /app

# Copy the compiled binary (package name defaults to repo name)
ARG BIN=logosV3

COPY --from=builder /app/target/release/${BIN} /app/bot

COPY . .
ENV LD_LIBRARY_PATH=/app/vendor/dectalk/dist
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1
USER app

# Run it!
CMD ["/app/bot"]
