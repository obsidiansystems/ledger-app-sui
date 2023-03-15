# General Nix Setup

Using Nix is the easiest way to develop Alamgu ledger apps, and we recommend you use it.

## Install Nix

Go to https://nixos.org/download.html and follow the instructions.
The multi-user install is recommended.

## Set up build caches (optional)

In addition to this app itself, the entire toolchain is packaged from source with Nix.
That means that with usuing a pre-populated source of build artifacts, the first build will take a **very long time** as everything specific to Alamgu is built from source.
(Other packages could also be built from source, but Nix by default ships configured to use the official `cache.nixos.org` build artifacts cache.)

If you are comfortable trusting Obsidian Systems's build farm, you can use our public open source cache for this purpose:

  - Store URL: `s3://obsidian-open-source`
  - Public key (for build artifact signatures): `obsidian-open-source:KP1UbL7OIibSjFo9/2tiHCYLm/gJMfy8Tim7+7P4o0I=`

To do this:

1. First you want to include these two in your `/etc/nix/nix.conf` settings file.
   After doing so, it should have two lines like this:
   ```
   substituters = https://cache.nixos.org/ s3://obsidian-open-source
   trusted-public-keys = cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY= obsidian-open-source:KP1UbL7OIibSjFo9/2tiHCYLm/gJMfy8Tim7+7P4o0I=
   ```
   (The new values are each appended at the end of a space-separated list.)

2. After updating that file, you probably need to restart your Nix daemon:

   - On macOS:
     - `sudo launchctl stop org.nixos.nix-daemon`
     - `sudo launchctl start org.nixos.nix-daemon`
   - On Linux:
     - `sudo systemctl stop nix-daemon`
     - `sudo systemctl start nix-daemon`

(On NixOS these tasks are done differently, consult the NixOS documentation for how to update your system configuration which includes these settings and will restart the daemon.)

### Notice for caching with "Apple Silicon" Macs

We don't currently do builds for "Apple Silicon" (i.e. `aarch64-darwin`).
To take advantage of our build caches then, you should pass `--system x86_64-darwin` to every `nix` invocation to instead use Intel mac builds instead.
