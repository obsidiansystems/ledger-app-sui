let self = import ./.;
in
  {
    inherit (self) tarball loadApp test;
    generic-cli = self.alamgu.generic-cli;
  }
