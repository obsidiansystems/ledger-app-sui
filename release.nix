let self = import ./.;
in
  {
    inherit (self) tarball loadApp test;
    generic-cli = self.ledger-platform.generic-cli;
  }
