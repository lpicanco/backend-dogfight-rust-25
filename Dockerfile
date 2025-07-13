FROM rust:1.88-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*


WORKDIR /app
COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo 'fn main() {}' > src/main.rs

RUN cargo build --release

COPY src src

RUN touch src/main.rs
RUN cargo build --release


FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/backend-dogfight-rust-25 /usr/local/bin/

CMD ["backend-dogfight-rust-25"]