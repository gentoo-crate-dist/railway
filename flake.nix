{
  description = "Travel with all your train information in one place";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils, ... }@inputs:
    (flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };
          name = "diebahn";
        in
        { 
          packages.default = 
            with pkgs;
            stdenv.mkDerivation rec {
              cargoDeps = rustPlatform.importCargoLock {
                lockFile = ./Cargo.lock;
                outputHashes = {
                  "hafas-rs-0.1.0" = "9YmWiief8Nux1ZkPTZjzer/qKAa5hORVn8HngMtKDxM=";
                };
              };
              src = ./.;
              buildInputs = with pkgs; [ libadwaita ];
              nativeBuildInputs = with pkgs; [ wrapGAppsHook4 rustPlatform.cargoSetupHook meson gettext glib pkg-config desktop-file-utils appstream-glib ninja rustc cargo ];

              inherit name;
            };
          devShells.default =
            let 
              run = pkgs.writeShellScriptBin "run" ''
                export GSETTINGS_SCHEMA_DIR=./build/data/
                meson compile -C build && ./build/target/debug/${name}
              '';
              check = pkgs.writeShellScriptBin "check" ''
                cargo clippy
              '';
            in
            with pkgs;
            pkgs.mkShell {
              src = ./.;
              buildInputs = with pkgs; [];
              nativeBuildInputs = with pkgs; [ wrapGAppsHook4 meson gettext glib gtk4 libadwaita pkg-config desktop-file-utils appstream-glib ninja rustc cargo clippy run check ];
              shellHook = ''
                meson setup -Dprofile=development build
              '';
            };
          apps.default = {
            type = "app";
            inherit name;
            program = "${self.packages.${system}.default}/bin/${name}";
          };
        })
    );
}
