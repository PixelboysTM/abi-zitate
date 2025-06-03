FROM rust:1.85-slim-bookworm AS chef
# Install OpenSSL and other dependencies
RUN apt-get update && apt-get install -y \
    curl \
    pkg-config \
    libssl-dev \
    ca-certificates \
    wget \
    gnupg \
    && echo "deb http://deb.debian.org/debian bullseye-backports main contrib" > /etc/apt/sources.list.d/backports.list \
    && apt-get update \
    && apt-get -y install -t bullseye-backports openssl

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

# Build the application
RUN dx build --release --features web
RUN ls -la /app/dist || true  # Debug: check if dist directory exists
RUN cargo build --release

FROM debian:bullseye-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates libssl-dev

# Copy the server binary and static files
COPY --from=builder /app/target/dx/abi-zitate/release/web/ /usr/local/app


# Set working directory and environment
WORKDIR /usr/local/app
ENV RUST_LOG=info
ENV PORT=8080
ENV HOST=0.0.0.0

EXPOSE 8080

CMD ["/usr/local/bin/server"]