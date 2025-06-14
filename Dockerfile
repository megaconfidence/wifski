# ---- Builder Stage ----
# Use a Rust image based on Debian Bullseye to ensure GLIBC compatibility
FROM rust:1-slim-bullseye AS builder

# Set the working directory
WORKDIR /usr/src/wifski-container

# Install build dependencies, including ca-certificates for SSL
RUN apt-get update && apt-get install -y libssl-dev pkg-config build-essential ca-certificates

# Copy the application source code
COPY ./wifski-container/ .

# Build the application in release mode
RUN cargo build --release

# ---- Final Stage ----
# Use Debian Bullseye as it has a newer GLIBC version compatible with the builder
FROM debian:bullseye-slim

# Install FFMPEG and runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends ffmpeg libssl1.1 && \
    rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/wifski-container/target/release/wifski-container /usr/local/bin/wifski-container

# Expose the port the application runs on
EXPOSE 8080

# Set the command to run the application
CMD ["wifski-container"]
