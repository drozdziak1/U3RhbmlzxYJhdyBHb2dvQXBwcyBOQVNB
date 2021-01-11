FROM ubuntu:latest

ENV DEBIAN_FRONTEND=noninteractive
RUN apt update
RUN apt install -y build-essential curl libssl-dev pkg-config

# Install Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

# Make Rust tools available in PATH
ENV PATH="/root/.cargo/bin:${PATH}"

# Check cargo is visible
RUN cargo --help
