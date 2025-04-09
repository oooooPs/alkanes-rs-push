FROM rust:1.85.1-slim-bullseye as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    git \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install wasm32-unknown-unknown target
RUN rustup target add wasm32-unknown-unknown

# Clone metashrew repository at specific branch
WORKDIR /usr/src
RUN git clone https://github.com/sandshrewmetaprotocols/metashrew.git -b v8.5.1-rc1
WORKDIR /usr/src/metashrew
RUN apt-get update && apt-get install -y \
    libssl-dev \
    libclang-dev \
    clang \
    pkg-config \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Build rockshrew-mono
RUN cargo build --release -p rockshrew-mono

# Copy and build alkanes-rs project
WORKDIR /usr/src
COPY . /usr/src/alkanes-rs
WORKDIR /usr/src/alkanes-rs

# Build alkanes with mainnet feature
RUN cargo build --release --target wasm32-unknown-unknown --features mainnet

# Create a smaller runtime image
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    libclang-dev \
    pkg-config \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create directory for RocksDB data
RUN mkdir -p /data/rocksdb

# Copy built binaries from builder stage
COPY --from=builder /usr/src/metashrew/target/release/rockshrew-mono /usr/local/bin/
COPY --from=builder /usr/src/alkanes-rs/target/wasm32-unknown-unknown/release/alkanes.wasm /usr/local/bin/

# Set working directory
WORKDIR /data

# Expose the port
EXPOSE 8080

# Set entrypoint
ENTRYPOINT ["rockshrew-mono", \
    "--db-path", "/data/rocksdb", \
    "--indexer", "/usr/local/bin/alkanes.wasm", \
    "--cors", "*", \
    "--host", "0.0.0.0", \
    "--port", "8080", \
    "--start-block", "880000"]

# Default command (can be overridden)
CMD []

# Usage:
# docker build -t alkanes-rs .
# docker run -p 8080:8080 -v /path/to/local/data:/data/rocksdb \
#   alkanes-rs --daemon-rpc-url http://bitcoind:8332 --auth rpcuser:rpcpassword
