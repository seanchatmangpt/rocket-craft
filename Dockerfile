# Stage 1: builder
# Builds the rocket-cmd binary from the tools/ workspace.
# Note: tools/ has some external path deps (wasm4pm-compat-stub is vendored inside
# the workspace; clap-noun-verb is on crates.io). We build only --bin rocket-cmd
# so any unreachable path deps in other crates don't block the build.
FROM rust:1.82-slim AS builder

# Install build deps needed by some crates (openssl, pkg-config, etc.)
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# ── Layer-cache trick: copy manifests first, build deps, then add source ──

# Copy workspace manifest + lockfile
COPY tools/Cargo.toml tools/Cargo.lock ./tools/

# Copy every member's Cargo.toml so cargo can resolve the workspace
COPY tools/rocket-cmd/Cargo.toml  ./tools/rocket-cmd/Cargo.toml
COPY tools/rocket-sdk/Cargo.toml  ./tools/rocket-sdk/Cargo.toml
COPY tools/knhk/Cargo.toml        ./tools/knhk/Cargo.toml
COPY tools/unrdf/Cargo.toml       ./tools/unrdf/Cargo.toml
COPY tools/un-test-utils/Cargo.toml ./tools/un-test-utils/Cargo.toml
COPY tools/wasm4pm-compat-stub/Cargo.toml ./tools/wasm4pm-compat-stub/Cargo.toml

# Create stub lib.rs files so cargo can build the dependency graph
RUN for crate in rocket-sdk knhk unrdf un-test-utils wasm4pm-compat-stub; do \
      mkdir -p tools/$crate/src && echo "// stub" > tools/$crate/src/lib.rs; \
    done && \
    mkdir -p tools/rocket-cmd/src && echo "fn main() {}" > tools/rocket-cmd/src/main.rs

WORKDIR /build/tools
# Pre-fetch / compile dependencies (layer cached unless Cargo.toml/lock changes)
RUN cargo build --release --bin rocket-cmd 2>/dev/null || true

# ── Now copy actual source and rebuild ──
WORKDIR /build
COPY tools/ ./tools/

WORKDIR /build/tools
RUN cargo build --release --bin rocket-cmd

# Stage 2: runtime
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary and expose it as `rocket`
COPY --from=builder /build/tools/target/release/rocket-cmd /usr/local/bin/rocket

# Default working directory is /workspace (mounted repo volume)
WORKDIR /workspace

ENTRYPOINT ["/usr/local/bin/rocket"]
CMD ["--help"]
