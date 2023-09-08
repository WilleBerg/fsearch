#!/bin/bash

echo "Checking for cargo..."
if ! which cargo > /dev/null 2>&1; then
    echo "Please install cargo (or add existing cargo to path)"
fi
echo "Building binary..."
mkdir ~/.fsearch
cargo build --release
if ! grep -q "$(pwd)/target/release" "/home/$USER/.bashrc"; then
    echo "PATH=\$PATH:\"$(pwd)/target/release\" # ADDED BY FSEARCH INSTALL SCRIPT AT $(date)" >> ~/.bashrc
    echo "Added fsearch binary to path"
    echo "Remember to source your .bashrc (. ~/.bashrc)"
fi
