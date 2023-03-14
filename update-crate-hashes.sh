#!/usr/bin/env bash

cd -- "$(dirname -- "${BASH_SOURCE[0]}")"

crate2nix="$(nix-build --no-out-link -A alamgu.crate2nix)/bin/crate2nix"
nix-shell -A nanosplus.rustShell --command "$crate2nix generate -f rust-app/Cargo.toml"
rm Cargo.nix
