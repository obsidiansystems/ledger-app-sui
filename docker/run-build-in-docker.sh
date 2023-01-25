#!/usr/bin/env bash
set -eu

OUT_DIR=/app/docker-outputs

RUST_NANOS_SDK=`mktemp -d`
TARGET_DIR=`mktemp -d`

git clone "$RUST_NANOS_SDK_GIT" "$RUST_NANOS_SDK"
cd "$RUST_NANOS_SDK"; git checkout "$RUST_NANOS_SDK_REV"; cd -

PATH=$RUST_NANOS_SDK/scripts:$PATH
export OBJCOPY="llvm-objcopy"
export NM="llvm-nm"

cd rust-app

for device in nanos nanosplus nanox
do
   cargo +nightly build --target-dir=$TARGET_DIR --release --target=$device.json -Z build-std=core
   cp $TARGET_DIR/$device/release/$APP_NAME $OUT_DIR/$device
   chown $HOST_UID:$HOST_GID $OUT_DIR/$device/$APP_NAME
done
