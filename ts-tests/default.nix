{ pkgs ? import (import ../dep/alamgu/thunk.nix + "/dep/nixpkgs") {}
, nodejs ? pkgs.nodejs
}:

let
  inherit (pkgs) lib;
  yarn2nix = import ../dep/yarn2nix { inherit pkgs; };
  inherit (import (import ../dep/alamgu/thunk.nix) {}) thunkSource;
  yarnDepsNix = pkgs.runCommand "yarn-deps.nix" {} ''
    ${yarn2nix}/bin/yarn2nix --offline \
      <(sed -e '/hw-app-obsidian-common/,/^$/d' ${./yarn.lock}) \
      > $out
  '';
  yarnPackageNix = pkgs.runCommand "yarn-package.nix" {} ''
    # We sed hw-app-obsidian-common to a constant here, so that the package.json can be whatever; we're overriding it anyways.
    ${yarn2nix}/bin/yarn2nix --template \
      <(sed 's/"hw-app-obsidian-common".*$/"hw-app-obsidian-common": "0.0.1",/' ${./package.json}) \
      > $out
  '';
  nixLib = yarn2nix.nixLib;

  localOverrides = self: super:
      let
        registries = {
          yarn = n: v: "https://registry.yarnpkg.com/${n}/-/${n}-${v}.tgz";
        };
        y = registries.yarn;
        s = self;
      in {
        "bcrypto@5.3.0" = super._buildNodePackage {
          key="bcrypto";
          version="5.3.0";
          src = pkgs.fetchurl {
            url = y "bcrypto" "5.3.0";
            sha1 = "d2d7d8a808b5efeb09fe529034a30bd772902d84";
          };
          buildPhase = ''
            ${pkgs.nodePackages.node-gyp}/bin/node-gyp rebuild --nodedir=${lib.getDev nodejs} # /include/node
          '';
         nativeBuildInputs = [ pkgs.python3 ];
          nodeBuildInputs = [
            (s."bufio@~1.0.7")
            (s."loady@~0.0.5")
          ];
        };

        # https://github.com/Profpatsch/yarn2nix/issues/56
        "char-regex@1.0.2" = {
          inherit (super."char-regex@1.0.2") key;
          drv = super."char-regex@1.0.2".drv.overrideAttrs (_: {
            dontMakeSourcesWritable = true;
            postUnpack = ''
              chmod +x $sourceRoot
              chmod -R +rw $sourceRoot
            '';
          });
        };

        "usb@1.8.8" = {
          inherit (super."usb@1.8.8") key;
          drv = super."usb@1.8.8".drv.overrideAttrs (attrs: {
            nativeBuildInputs = [ pkgs.python3 pkgs.systemd pkgs.v8 nodejs pkgs.libusb1 ];
            dontBuild = false;
            buildPhase = ''
              ln -s ${nixLib.linkNodeDeps { name=attrs.name; dependencies=attrs.passthru.nodeBuildInputs; }} node_modules
              ${pkgs.nodePackages.node-gyp}/bin/node-gyp rebuild --nodedir=${lib.getDev nodejs} # /include/node
            '';
          });
        };

        "node-hid@1.3.0" = {
          inherit (super."node-hid@1.3.0") key;
          drv = super."node-hid@1.3.0".drv.overrideAttrs (attrs: {
            nativeBuildInputs = [ pkgs.python3 pkgs.systemd pkgs.v8 nodejs pkgs.libusb1 pkgs.pkg-config ];
            dontBuild = false;
            buildPhase = ''
              ln -s ${nixLib.linkNodeDeps { name=attrs.name; dependencies=attrs.passthru.nodeBuildInputs; }} node_modules
              ${pkgs.nodePackages.node-gyp}/bin/node-gyp rebuild --nodedir=${lib.getDev nodejs} # /include/node
            '';
          });
        };

        "hw-app-obsidian-common@0.0.1" = super._buildNodePackage rec {
          key = "hw-app-obsidian-common";
          version = "0.0.1";
          src = thunkSource ../dep/hw-app-obsidian-common;
          buildPhase = ''
            ln -s $nodeModules node_modules
            node $nodeModules/.bin/tsc
            node $nodeModules/.bin/tsc -m ES6 --outDir lib-es
          '';
          nodeModules = nixLib.linkNodeDeps {
            name = "hw-app-obsidian-common";
            dependencies = nodeBuildInputs ++ [
              (s."@types/node@^16.10.3")
              (s."typescript@^4.4.3")
            ];
          };
          passthru = { inherit nodeModules; };
          NODE_PATH = nodeModules;
          nodeBuildInputs = [
            (s."@ledgerhq/hw-transport@^6.3.0")
            (s."fast-sha256@^1.3.0")
            (s."typedoc@^0.22.7")
          ];
        };

      };

  deps = nixLib.buildNodeDeps
    (lib.composeExtensions
      (pkgs.callPackage yarnDepsNix {
        fetchgit = builtins.fetchGit;
      })
      localOverrides);

  src0 = lib.sources.cleanSourceWith {
    src = ./.;
    filter = p: _: let
      p' = baseNameOf p;
      srcStr = builtins.toString ./.;
    in p' != "node_modules";
  };

  src = lib.sources.sourceFilesBySuffices src0 [
    ".js" ".cjs" ".ts" ".json"
  ];
in rec {
  inherit deps yarnDepsNix yarnPackageNix thunkSource;

  testModules = nixLib.buildNodePackage ({
    src = pkgs.runCommand "package-json" {} ''
      mkdir $out
      cp ${./package.json} $out/package.json
    '';
  } // nixLib.callTemplate yarnPackageNix deps);

  testScript = pkgs.writeShellScriptBin "mocha-wrapper" ''
    set -e
    cd ${testPackage}
    export NODE_PATH=${testPackage}/node_modules
    export NO_UPDATE_NOTIFIER=true
    exec ${pkgs.yarn}/bin/yarn --offline run test
  '';

  testPackage = nixLib.buildNodePackage ({
    inherit src;
  } // nixLib.callTemplate yarnPackageNix deps);
}
