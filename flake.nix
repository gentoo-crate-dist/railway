{
  description = "Travel with all your train information in one place";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.nixpkgsgnome.url = "github:NixOS/nixpkgs/gnome";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, nixpkgsgnome, flake-utils, ... }@inputs:
    (flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };
          pkgsgnome = import nixpkgsgnome {
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
              buildInputs = [ pkgsgnome.libadwaita pkgsgnome.gtk4 ];
              nativeBuildInputs = [ pkgsgnome.wrapGAppsHook4 pkgs.rustPlatform.cargoSetupHook pkgs.meson pkgs.gettext pkgsgnome.glib pkgs.pkg-config pkgs.desktop-file-utils pkgs.appstream-glib pkgs.ninja pkgs.rustc pkgscargo ];

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
              buildInputs = [];
              nativeBuildInputs = [ pkgsgnome.wrapGAppsHook4 pkgs.meson pkgs.gettext pkgsgnome.glib pkgsgnome.gtk4 pkgsgnome.libadwaita pkgs.pkg-config pkgs.desktop-file-utils pkgs.appstream-glib pkgs.ninja pkgs.rustc pkgs.cargo pkgs.clippy run check ];
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
