{pkgs ? import <nixpkgs> {
    inherit system;
  }, system ? builtins.currentSystem, nodejs ? pkgs."nodejs-14_x"}:

let
  nodeEnvOrig = import ./node-env.nix {
    inherit (pkgs) stdenv lib python2 runCommand writeTextFile;
    inherit pkgs nodejs;
    libtool = if pkgs.stdenv.isDarwin then pkgs.darwin.cctools else null;
  };

  nodeEnv = nodeEnvOrig // {
    buildNodeDependencies = nodeEnvOrig.buildNodeDependencies.overrideAttrs (attrs: {
      installPhase = ''
        exit 1
      '';
      postInstall = postInstallFixup;
      # installPhase = attrs.installPhase + postInstallFixup;
    });
  };
  nodePackages = import ./node-packages.nix {
    inherit (pkgs) fetchurl nix-gitignore stdenv lib fetchgit;
    inherit nodeEnv;
  };
  postInstallFixup = ''
      # Fixup for checkouts from git.
        echo "POST INSTALL FIXUP RUNNING"
        for pkg in hw-app-obsidian-common
        do
        if [ -d node_modules/$pkg ]
        then pushd node_modules/$pkg; npm run prepare; popd;
        fi
        done
  '';
in
  nodePackages // {
    package = nodePackages.package.override {
      postInstall = postInstallFixup;
    };
    shell = (nodePackages.shell.override {
          preFixup = ''
          pushd $out/lib/
          ${postInstallFixup}
          popd
          '';
        }).overrideAttrs (attrs: {
          shellHook = attrs.shellHook + ''
            export TS_NODE_COMPILER_OPTIONS="{\"baseUrl\": \"$NODE_PATH\"}"
          '';
          ATTRS=builtins.toJSON attrs;
        });
      }
