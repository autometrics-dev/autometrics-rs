FROM gitpod/workspace-full:latest

# Install Prometheus
RUN bash -c "brew install prometheus"

# Install a recent rust version
RUN bash -c "rustup toolchain install 1.65.0"
# Install 'cargo watch' for better dev experience
RUN bash -c "cargo install cargo-watch"
