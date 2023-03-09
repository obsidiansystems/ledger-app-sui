let
  alamgu-path = import ./dep/alamgu/thunk.nix;
  pkgsSrc = import (alamgu-path + "/dep/nixpkgs/thunk.nix");
  lib = import (pkgsSrc + "/lib");

  x86_64-linux = import ./. rec {
    localSystem = { system = "x86_64-linux"; };
  };
  x86_64-darwin = import ./. rec {
    localSystem = { system = "x86_64-darwin"; };
  };
in {
  inherit x86_64-linux x86_64-darwin;
}
  # Hack until CI will traverse contents
  // lib.mapAttrs' (n: lib.nameValuePair ("linux--" + n)) x86_64-linux
  // lib.mapAttrs' (n: lib.nameValuePair ("linux--nanos--" + n)) x86_64-linux.nanos
  // lib.mapAttrs' (n: lib.nameValuePair ("linux--nanox--" + n)) x86_64-linux.nanox
  // lib.mapAttrs' (n: lib.nameValuePair ("linux--nanosplus--" + n)) x86_64-linux.nanosplus
  // lib.mapAttrs' (n: lib.nameValuePair ("macos--" + n)) x86_64-darwin
  // lib.mapAttrs' (n: lib.nameValuePair ("macos--nanos--" + n)) x86_64-darwin.nanos
  // lib.mapAttrs' (n: lib.nameValuePair ("macos--nanox--" + n)) x86_64-darwin.nanox
  // lib.mapAttrs' (n: lib.nameValuePair ("macos--nanosplus--" + n)) x86_64-darwin.nanosplus
