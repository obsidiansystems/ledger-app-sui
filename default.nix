rec {
  ledger-platform = import ./dep/ledger-platform {};

  inherit (ledger-platform)
    pkgs ledgerPkgs
    crate2nix
    buildRustCrateForPkgsLedger;

  app = import ./Cargo.nix {
    pkgs = ledgerPkgs;
    buildRustCrateForPkgs = pkgs: (buildRustCrateForPkgsLedger pkgs).override {
      defaultCrateOverrides = pkgs.defaultCrateOverrides // {
        nanos_sdk = _: {
          RUSTC_BOOTSTRAP = true;
        };
        rust-app = attrs: {
          preHook = ledger-platform.gccLibsPreHook;
          extraRustcOpts = [
            "-C" "relocation-model=ropi"
            "-C" "link-arg=-T${(builtins.elemAt attrs.dependencies 0).lib}/lib/nanos_sdk.out/script.ld"
          ];
        };
      };
    };
  };
}
