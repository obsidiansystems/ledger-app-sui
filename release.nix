let self = import ./.;
    lib = self.pkgs.lib;
in
  {
    generic-cli = self.alamgu.generic-cli;
  }
  // lib.mapAttrs' (n: lib.nameValuePair ("nanos--" + n)) self.nanos
  // lib.mapAttrs' (n: lib.nameValuePair ("nanox--" + n)) self.nanox
  // lib.mapAttrs' (n: lib.nameValuePair ("nanosplus--" + n)) self.nanosplus
