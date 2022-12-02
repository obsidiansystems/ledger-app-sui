let
  tarballNix = import ./.;
in
  tarballNix.alamgu.pkgs.mkShell {
    buildInputs = [ tarballNix.load-app ];
  }
