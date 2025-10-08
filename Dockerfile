########## Builder ##########
FROM rust:1.90-bookworm AS builder

WORKDIR /app

# Bindgen / libclang for build.rs
RUN apt-get update && apt-get install -y \
    clang llvm-dev libclang-dev pkg-config build-essential ca-certificates \

    && rm -rf /var/lib/apt/lists/*

#  Cache deps
COPY Cargo.toml Cargo.lock ./
# Warm the cache with a dummy main so dependency compile is cached
RUN mkdir -p src && echo 'fn main(){}' > src/main.rs && \
    cargo build --release || true

# Copy the rest of the source tree (including your src/, s.json, etc.)
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
