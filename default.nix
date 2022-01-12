rec {
  ledger-platform = import ./dep/ledger-platform {};

  inherit (ledger-platform)
    lib
    pkgs ledgerPkgs
    crate2nix
    buildRustCrateForPkgsLedger;

  app = import ./Cargo.nix {
    pkgs = ledgerPkgs;
    buildRustCrateForPkgs = pkgs: let
      fun = (buildRustCrateForPkgsLedger pkgs).override {
        defaultCrateOverrides = pkgs.defaultCrateOverrides // {
          nanos_sdk = _: {
            RUSTC_BOOTSTRAP = true;
          };
          ledger-parser-combinators = _: {
            RUSTC_BOOTSTRAP = true;
          };
          rust-app = attrs: let
            sdk = lib.findFirst (p: lib.hasPrefix "rust_nanos_sdk" p.name) (builtins.throw "no sdk!") attrs.dependencies;
          in {
            preHook = ledger-platform.gccLibsPreHook;
            extraRustcOpts = [
              "-C" "relocation-model=ropi"
              "-C" "link-arg=-T${sdk.lib}/lib/nanos_sdk.out/script.ld"
              "-C" "linker=${pkgs.stdenv.cc.targetPrefix}lld"
            ];
          };
        };
      };
    in
      args: fun (args // lib.optionalAttrs pkgs.stdenv.hostPlatform.isAarch32 {
        dependencies = map (d: d // { stdlib = true; }) [
          ledger-platform.ledgerCore
          ledger-platform.ledgerCompilerBuiltins
        ] ++ args.dependencies;
      });
  };

  # For CI
  rootCrate = app.rootCrate.build;
}
