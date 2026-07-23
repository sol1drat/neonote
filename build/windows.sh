#!/bin/sh
set -e

echo "starting isolated Nix environment"

nix-shell --run "cargo build --target x86_64-pc-windows-gnu --release"

echo "build successful"
echo "location: target/x86_64-pc-windows-gnu/release/neonote.exe"
