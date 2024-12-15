FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    sqlite3 \
    libsqlite3-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Copy Cargo files first to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Create a minimal src/main.rs for dependency checking
RUN mkdir src && \
    echo "fn main() {println!(\"dummy\");}" > src/main.rs && \
    cargo check && \
    rm -rf src/


# Now copy the real source code
COPY . .

# Build the actual application
RUN cargo build --release

FROM debian:bullseye-slim
WORKDIR /usr/src/app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y \
    sqlite3 \
    libsqlite3-0 \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/yrealestate_backend .
COPY --from=builder /usr/src/app/migrations ./migrations
COPY --from=builder /usr/src/app/.env .

# Install SQLite
RUN apt-get update && apt-get install -y sqlite3 && rm -rf /var/lib/apt/lists/*

# Create db directory
RUN mkdir -p db

EXPOSE 8080
CMD ["./yrealestate_backend"]