{
  description = "Minimum Viewer - Rust CUI file viewer";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "minimum-viewer";
          version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          # These tests call the system pbcopy, which is not on PATH in the Nix build sandbox.
          checkFlags = [
            "--skip=command::yank::tests::yank_file_copies_path_and_sets_yanked_message"
            "--skip=command::yank::tests::yank_directory_copies_path_and_sets_yanked_message"
          ];
          meta.mainProgram = "mmv";
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
            rust-analyzer
          ];
        };
      });
}
