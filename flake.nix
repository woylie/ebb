# SPDX-FileCopyrightText: 2025 Mathias Polligkeit
#
# SPDX-License-Identifier: AGPL-3.0-or-later

{
  description = "CLI application for time tracking and flex time balance";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
        src = ./.;
        cargoLock = {
          lockFile = ./Cargo.lock;
        };

        overlays = [ (import rust-overlay) ];

        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable."1.99.0".default;
        rustPlatform = pkgs.rustPlatform;

        packages = {
          default = rustPlatform.buildRustPackage {
            inherit version src cargoLock;
            pname = "ebb";
            buildInputs = [ ];
            nativeBuildInputs = [ pkgs.installShellFiles ];

            postInstall = pkgs.lib.optionalString (pkgs.stdenv.buildPlatform.canExecute pkgs.stdenv.hostPlatform) ''
              installShellCompletion --cmd ebb \
                --bash <($out/bin/ebb generate-completions --shell bash) \
                --fish <($out/bin/ebb generate-completions --shell fish) \
                --zsh <($out/bin/ebb generate-completions --shell zsh)
            '';
          };
        };
      in
      {
        inherit packages;

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            mdbook
            nixfmt
            prettier
            reuse
            rustToolchain
          ];
        };

        checks = {
          formatting = rustPlatform.buildRustPackage {
            pname = "ebb-formatting";
            inherit version src cargoLock;

            nativeBuildInputs = with pkgs; [
              nixfmt
              prettier
              rustToolchain
            ];

            buildPhase = ''
              cargo fmt --all --check
              nixfmt -c flake.nix
              prettier **/*.{md,json,toml,yml} --check
            '';

            installPhase = "mkdir -p $out";
            doCheck = false;
          };

          lint = rustPlatform.buildRustPackage {
            pname = "ebb-lint";
            inherit version src cargoLock;

            nativeBuildInputs = with pkgs; [ rustToolchain ];

            buildPhase = ''
              cargo clippy --all-targets --all-features --no-deps
            '';

            installPhase = "mkdir -p $out";
            doCheck = false;
          };

          licenses = rustPlatform.buildRustPackage {
            pname = "ebb-licenses";
            inherit version src cargoLock;

            nativeBuildInputs = with pkgs; [
              cargo-deny
              reuse
            ];

            buildPhase = ''
              cargo deny check licenses
              reuse lint
            '';

            installPhase = "mkdir -p $out";
            doCheck = false;
          };

          test = packages.default;
        };
      }
    );
}
