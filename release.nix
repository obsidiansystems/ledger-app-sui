let self = import ./.;
    lib = self.pkgs.lib;
in
  {
    generic-cli = self.alamgu.generic-cli;
  }
  // lib.mapAttrs' (n: lib.nameValuePair ("nanos--" + n)) (builtins.removeAttrs self.nanos ["test-with-logging" "stack-check-with-logging"])
  // lib.mapAttrs' (n: lib.nameValuePair ("nanox--" + n)) self.nanox
  // lib.mapAttrs' (n: lib.nameValuePair ("nanosplus--" + n)) self.nanosplus
