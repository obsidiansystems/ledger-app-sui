#!/usr/bin/env bash

# Use this script when updating the ledger_device_sdk, to regenerate bindings.rs and C SDK thunk

set -eu

nix-thunk unpack ./dep/ledger_secure_sdk_sys-bindings

for device in nanos nanosplus nanox
do
    export DEVICE=$device
    nix-shell -A $DEVICE.rustShell --run ' \
      set -x
      cd rust-app; \
      cargo clean; \
      cargo build --target=$TARGET_JSON; \
      C_SDK_PATH=`find ./target/$DEVICE -type d -name 'ledger-secure-sdk'`; \
      C_SDK_HASH=`git -C $C_SDK_PATH describe --always --exclude \* --abbrev=40`; \
      C_SDK_URL=`git -C $C_SDK_PATH config --get remote.origin.url`; \
      BINDINGS_PATH=`find ./target/$DEVICE -type f -name 'bindings.rs'`; \
      mkdir -p ../dep/ledger_secure_sdk_sys-bindings/$DEVICE; \
      cp "$BINDINGS_PATH" "../dep/ledger_secure_sdk_sys-bindings/$DEVICE/"; \
      rm -r ../dep/ledger-secure-sdk-$DEVICE; \
      nix-thunk create "$C_SDK_URL" ../dep/ledger-secure-sdk-$DEVICE --rev $C_SDK_HASH;
    '
done

# This should fail in case of update to the bindings
# Review the changes with the nix-build; then commit, push and pack
nix-thunk pack ./dep/ledger_secure_sdk_sys-bindings
