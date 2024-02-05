#!/usr/bin/env bash
set -eu

OUT_DIR=/app/docker-outputs

RUST_NANOS_SDK=`mktemp -d`
TARGET_DIR=`mktemp -d`

git clone "$RUST_NANOS_SDK_GIT" "$RUST_NANOS_SDK"
cd "$RUST_NANOS_SDK"; git checkout "$RUST_NANOS_SDK_REV"; cd -

# stat present in docker does not recognize --format
sed -i 's/stat --format/stat -c/' $RUST_NANOS_SDK/ledger_device_sdk/scripts/link_wrap.sh

PATH=$RUST_NANOS_SDK/ledger_device_sdk/scripts:$PATH
export OBJCOPY="llvm-objcopy"
export NM="llvm-nm"

cd rust-app

# $RUST_NIGHTLY is set in ledger-app-builder
for device in nanos nanosplus nanox
do
   cargo +$RUST_NIGHTLY build --target-dir=$TARGET_DIR --release --target=$device.json -Z build-std=core
   cp $TARGET_DIR/$device/release/$APP_NAME $OUT_DIR/$device
   chown $HOST_UID:$HOST_GID $OUT_DIR/$device/$APP_NAME
done
