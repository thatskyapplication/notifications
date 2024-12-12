FROM rust:1-bookworm as builder
WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim

# Install necessary dependencies.
RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

RUN useradd -ms /bin/bash appuser
WORKDIR /app
COPY --from=builder /usr/src/app/target/release/caelus-notifications /app/caelus-notifications
RUN chmod +x /app/caelus-notifications
USER appuser
ENV RUST_ENV=production
CMD ["./caelus-notifications"]
