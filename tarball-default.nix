rec {
  alamgu = import (import ./dep/alamgu/thunk.nix) {};
  ledgerctl = alamgu.ledgerctl;
  this = ./.;
  load-app = alamgu.pkgs.writeScriptBin "load-app" ''
    #!/usr/bin/env bash

    cd ${this}
    ${ledgerctl}/bin/ledgerctl install -f ${this}/app.json
  '';
}
