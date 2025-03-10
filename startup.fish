#!/usr/bin/fish
sudo systemctl start docker
sudo docker compose -f dev.compose.yaml up -d
cargo run --release
