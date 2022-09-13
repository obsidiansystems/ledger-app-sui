# Rust Ledger Nano S, Nano S+ and Nano X Application

This application is compatible with
- Ledger Nano S, running FW 2.1.0 and above
- Ledger Nano S+, running FW 1.0.3
- Ledger Nano X

Note: the compatibility with Ledger Nano X has only been checked on Speculos emulator

### Installation using the pre-packaged tarball

Before installing please ensure that your device is plugged, unlocked, and on the device home screen.
Installing the app from a tarball can be done using `ledgerctl`.

By using Nix, this can be done simply by using the `load-app` command, without manually installing the `ledgerctl` on your system.

```
tar xzf release.tar.gz
cd rust-app
nix-shell
load-app
```

Without using Nix, the `ledgerctl` can be used directly to install the app with the following commands.
For more information on how to install and use that tool see the [instructions from LedgerHQ](https://github.com/LedgerHQ/ledgerctl).

```bash
tar xzf release.tar.gz
cd rust-app
ledgerctl install -f app.json
```

## Using the app with generic CLI tool

The bundled `generic-cli` tool can be used to obtaining the public key and do signing.

To use this tool using Nix, from the root level of this repo, run:

```
nix-shell -A nanos.appShell

generic-cli getAddress "44'/535348'/0'/0/0"

generic-cli sign "44'/535348'/0'/0/0" --json '{"chain_id":"testnet","entropy":"-7780543831205109370","fee":[{"amount":"10000","denom":"upokt"}],"memo":"","msg":{"type":"pos/Send","value":{"amount":"1000000","from_address":"51568b979c4c017735a743e289dd862987143290","to_address":"51568b979c4c017735a743e289dd862987143290"}}}'
```

## Building the app from source

This application has been packaged up with [Nix](https://nixos.org/).

### Nix/Linux

Using Nix, from the root level of this repo, run:

```bash
nix-shell -A alamgu.rustShell
cd rust-app/
# For NanoS+, replace nanos with nanosplus. It is currently not possible to load the app on Nano X
cargo-ledger ledger -l nanos
````

The [cargo-ledger](https://github.com/LedgerHQ/cargo-ledger.git) builds, outputs a `hex` file and a manifest file for `ledgerctl`, and loads it on a device in a single `cargo-ledger ledger -l nanos` command in the rust-app folder within app directory.

You do not need to install cargo-ledger outside of the nix-shell.

Before installing, please ensure that your device is plugged, unlocked, and on the device home screen.

## Running tests

Using Nix, from the root level of this repo, run:

```bash
nix-shell -A alamgu.rustShell
cd rust-app/
cargo test --target=nanos.json
cargo test --target=nanosplus.json
cargo test --target=nanox.json
````
