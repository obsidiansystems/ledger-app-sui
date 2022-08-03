let
  ledgerPlatform = import (fetchTarball "https://github.com/alamgu/alamgu/archive/develop.tar.gz") {};
  pkgs = ledgerPlatform.pkgs;
  load-app = import ./.;
in
  pkgs.mkShell {
    buildInputs = [load-app];
  }
