# Combined Dockerfile - Backend + Frontend in ONE container
# This is the CHEAPEST way to deploy!

# Stage 1: Build Rust backend (static musl binary for Alpine)
FROM rust:latest AS backend-builder
WORKDIR /backend

# Install musl tools for static linking
RUN apt-get update && apt-get install -y musl-tools && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-unknown-linux-musl

COPY backend/Cargo.toml ./
COPY backend/src ./src

# Build static binary
RUN cargo build --release --target x86_64-unknown-linux-musl

# Stage 2: Build WASM frontend
FROM rust:latest AS frontend-builder
WORKDIR /app

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Copy frontend code
COPY Cargo.toml ./
COPY src ./src
COPY index.html ./

# Build WASM
RUN wasm-pack build --target web --release

# Stage 3: Runtime - Nginx serves frontend + Backend runs alongside
FROM nginx:alpine

# Install runtime dependencies for backend
RUN apk add --no-cache ca-certificates supervisor

# Copy static backend binary (musl, works natively on Alpine!)
COPY --from=backend-builder /backend/target/x86_64-unknown-linux-musl/release/game_server /usr/local/bin/game_server
RUN chmod +x /usr/local/bin/game_server

# Copy frontend files
COPY --from=frontend-builder /app/index.html /usr/share/nginx/html/
COPY --from=frontend-builder /app/pkg /usr/share/nginx/html/pkg

# Supervisor config to run BOTH services
RUN mkdir -p /etc/supervisor.d
COPY supervisord.conf /etc/supervisord.conf

# Expose ports (80 for frontend, 9001 for backend)
EXPOSE 80 9001

# Start supervisor (manages both nginx and backend)
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisord.conf"]

