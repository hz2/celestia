{
  description = "no_std sun and moon position/phase calculations";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      crane,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        rustToolchain = pkgs.rust-bin.stable."1.98.0".default.override {
          extensions = [
            "rust-src"
            "clippy"
            "rustfmt"
          ];
          targets = [ "thumbv7em-none-eabi" ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Keep in sync with `rust-version` in Cargo.toml.
        msrvToolchain = pkgs.rust-bin.stable."1.86.0".default;
        craneLibMsrv = (crane.mkLib pkgs).overrideToolchain msrvToolchain;

        src = craneLib.cleanCargoSource ./.;

        commonArgs = {
          inherit src;
          strictDeps = true;
          pname = "celestia";
          version = "0.1.0";
        };

        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // { cargoExtraArgs = "--features std"; });

        celestia = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
            cargoExtraArgs = "--features std";
          }
        );
      in
      {
        packages = {
          default = celestia;
          inherit celestia;
        };

        checks = {
          inherit celestia;

          tests = craneLib.cargoTest (commonArgs // { inherit cargoArtifacts; cargoExtraArgs = "--features std"; });

          clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets --features std -- -D warnings";
            }
          );

          fmt = craneLib.cargoFmt { inherit src; };

          docs = craneLib.mkCargoDerivation (
            commonArgs
            // {
              inherit cargoArtifacts;
              pnameSuffix = "-docs";
              buildPhaseCargoCommand = "RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --features std";
              installPhaseCommand = "mkdir -p $out";
            }
          );

          msrv = craneLibMsrv.cargoTest (
            commonArgs
            // {
              cargoArtifacts = craneLibMsrv.buildDepsOnly (
                commonArgs // { cargoExtraArgs = "--features std"; }
              );
              cargoExtraArgs = "--features std";
            }
          );

          # the whole point of this crate: verify it actually builds
          # no_std + libm for an embedded target, not just --no-default-features
          # on the host triple.
          no-std-verify = craneLib.mkCargoDerivation (
            commonArgs
            // {
              inherit cargoArtifacts;
              pnameSuffix = "-nostd-verify";
              buildPhaseCargoCommand = "cargo build --no-default-features --features libm --target thumbv7em-none-eabi";
              installPhaseCommand = "mkdir -p $out";
            }
          );
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ celestia ];
          packages = [
            rustToolchain
            pkgs.rust-analyzer
            pkgs.cargo-outdated
            pkgs.cargo-machete
            pkgs.gappa
            # cargo-kani isn't packaged in nixpkgs; install separately
            # with `cargo install --locked kani-verifier && cargo kani setup`.
          ];
        };
      }
    );
}
