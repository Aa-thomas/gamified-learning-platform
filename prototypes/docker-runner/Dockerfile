# Rust sandbox for running student code safely
FROM rust:1.75-slim

# Install clippy for linting
RUN rustup component add clippy

# Create a non-root user for running student code
RUN useradd -m -u 1000 student

# Set working directory
WORKDIR /challenge

# Set ownership to student user
RUN chown -R student:student /challenge

# Switch to non-root user
USER student

# Set resource limits (enforced by docker run flags)
# --memory=256m --cpus=1.0 --network=none

# Default command - will be overridden by runner
CMD ["cargo", "test"]
