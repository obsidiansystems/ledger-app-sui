rec {
  alamgu = import (fetchTarball "https://github.com/alamgu/alamgu/archive/main.tar.gz") {};
  ledgerctl = alamgu.ledgerctl;
  this = ./.;
  load-app = alamgu.pkgs.writeScriptBin "load-app" ''
    #!/usr/bin/env bash

    cd ${this}
    ${ledgerctl}/bin/ledgerctl install -f ${this}/app.json
  '';
}
