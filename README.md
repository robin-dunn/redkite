# redkite

Rust HTTP proxy

## Setup dev environment

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. "$HOME/.cargo/env"
rustup target add aarch64-unknown-linux-musl
rustup target add x86_64-unknown-linux-musl