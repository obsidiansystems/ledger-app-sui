#!/usr/bin/env bash
set -eu

export APP_NAME=`grep name rust-app/Cargo.toml | cut -d '"' -f2 | head -n1`
export RUST_NANOS_SDK_REV=`grep ledger-nanos-sdk rust-app/Cargo.lock | cut -d '"' -f2 | cut -d '#' -f2`
export RUST_NANOS_SDK_GIT=`grep ledger-nanos-sdk rust-app/Cargo.lock | cut -d '?' -f1 | cut -d '+' -f2`

OUT_DIR="./docker-outputs"
for device in nanos nanosplus nanox
do
    mkdir -p $OUT_DIR/$device
done

# Build apps using nightly
docker run \
  --env APP_NAME \
  --env RUST_NANOS_SDK_REV \
  --env RUST_NANOS_SDK_GIT \
  --env HOST_UID=$(id -u) \
  --env HOST_GID=$(id -g) \
  --rm -ti -v "$(realpath .):/app" \
  ghcr.io/ledgerhq/ledger-app-builder/ledger-app-builder:latest \
  docker/run-build-in-docker.sh

# Run tests
# The speculos-wrapper need to be invoked from a dir further down, as it refers to "../ts-tests"
nix-shell -A nanos.rustShell --run "cd $OUT_DIR; ../speculos-wrapper -m nanos ../$OUT_DIR/nanos/$APP_NAME"
nix-shell -A nanosplus.rustShell --run "cd $OUT_DIR; ../speculos-wrapper -m nanosp -a 1 ../$OUT_DIR/nanosplus/$APP_NAME"
nix-shell -A nanox.rustShell --run "cd $OUT_DIR; ../speculos-wrapper -m nanox -a 5 ../$OUT_DIR/nanox/$APP_NAME"

echo "Tests done!"

# Create app.hex
for device in nanos nanosplus nanox
do
    cp rust-app/Cargo.toml $OUT_DIR/$device/
    cp rust-app/*.gif $OUT_DIR/$device/
    nix-shell -A alamgu.perDevice.$device.rustShell --run "cd $OUT_DIR/$device; cargo ledger --use-prebuilt $APP_NAME --hex-next-to-json build $device"
done

echo "Use the following commands to install app"
echo 'nix-shell -A alamgu.rustShell --run "cd docker-outputs/nanos; ledgerctl install -f app_nanos.json"'
echo 'nix-shell -A alamgu.rustShell --run "cd docker-outputs/nanosplus; ledgerctl install -f app_nanosplus.json"'
