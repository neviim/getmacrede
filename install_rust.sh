#!/bin/bash

# Update package list
echo "Updating package list..."
sudo apt update

# Install dependencies (curl and build tools)
echo "Installing dependencies..."
sudo apt install -y curl build-essential

# Install Rust using rustup
echo "Installing Rust..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Configure the current shell
echo "Configuring shell..."
. "$HOME/.cargo/env"

echo "Rust installation complete!"
rustc --version
cargo --version
