FROM rust:1.85-slim-bookworm AS chef
# Install OpenSSL and other dependencies
RUN apt-get update && apt-get install -y \
    curl \
    pkg-config \
    libssl-dev \
    ca-certificates \
    wget \
    gnupg

RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .

# Install Dioxus CLI
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall dioxus-cli --root /.cargo -y --force
ENV PATH="/.cargo/bin:$PATH"

# Build the fullstack application
RUN dx bundle --release

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates libssl3

# Copy the bundled application
COPY --from=builder /app/target/dx/abi-zitate/release/web /usr/local/app/

# Set working directory and environment
WORKDIR /usr/local/app
ENV RUST_LOG=info
ENV PORT=8080
ENV HOST=0.0.0.0
ENV RUST_BACKTRACE=1

EXPOSE 8080

# Run the server binary that was created by dx bundle
CMD ["/usr/local/app/server"]