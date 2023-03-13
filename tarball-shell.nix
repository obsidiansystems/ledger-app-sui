let
  tarballNix = import ./.;
in
  tarballNix.alamgu.pkgs.mkShell {
    nativeBuildInputs = [ tarballNix.load-app ];
    strictDeps = true;
  }
