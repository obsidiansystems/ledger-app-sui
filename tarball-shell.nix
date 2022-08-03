let
  alamgu = import (fetchTarball "https://github.com/alamgu/alamgu/archive/develop.tar.gz") {};
  pkgs = alamgu.pkgs;
  load-app = import ./.;
in
  pkgs.mkShell {
    buildInputs = [load-app];
  }
