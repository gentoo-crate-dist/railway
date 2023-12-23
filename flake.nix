{
  description = "Find all your travel information";

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
              };
              src = let fs = lib.fileset; in fs.toSource {
                root = ./.;
                fileset =
                  fs.difference
                    ./.
                    (fs.unions [
                      (fs.maybeMissing ./result)
                      (fs.maybeMissing ./build)
                      ./flake.nix
                      ./flake.lock
                    ]);
              };
              buildInputs = [ pkgs.libadwaita pkgs.gtk4 ];
              nativeBuildInputs = [ pkgs.wrapGAppsHook4 pkgs.rustPlatform.cargoSetupHook pkgs.meson pkgs.gettext pkgs.glib pkgs.pkg-config pkgs.desktop-file-utils pkgs.appstream-glib pkgs.ninja pkgs.rustc pkgs.cargo ];

              inherit name;
            };
          devShells.default =
            let 
              run = pkgs.writeShellScriptBin "run" ''
                export GSETTINGS_SCHEMA_DIR=${pkgs.gtk4}/share/gsettings-schemas/${pkgs.gtk4.name}/glib-2.0/schemas/:${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}/glib-2.0/schemas/:./build/data/
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
              nativeBuildInputs = [ pkgs.wrapGAppsHook4 pkgs.meson pkgs.gettext pkgs.glib pkgs.gtk4 pkgs.libadwaita pkgs.pkg-config pkgs.desktop-file-utils pkgs.appstream-glib pkgs.ninja pkgs.rustc pkgs.cargo pkgs.clippy run check ];
              shellHook = ''
                meson setup -Dprofile=development build
              '';
            };
          apps.default = {
            type = "app";
            inherit name;
            program = "${self.packages.${system}.default}/bin/${name}";
          };

          # Note: This may only be run interactively as this requires network access.
          packages.makeScreenshot =
            let
              nixos-lib = import (nixpkgs + "/nixos/lib") { };
            in
            nixos-lib.runTest {
              name = "screenshot";
              hostPkgs = pkgs;
              imports = [
                {
                  nodes = {
                    machine = { pkgs, ... }: {
                      boot.loader.systemd-boot.enable = true;
                      boot.loader.efi.canTouchEfiVariables = true;

                      services.xserver.enable = true;
                      services.xserver.displayManager.gdm.enable = true;
                      services.xserver.desktopManager.gnome.enable = true;
                      services.xserver.displayManager.autoLogin.enable = true;
                      services.xserver.displayManager.autoLogin.user = "alice";

                      users.users.alice = {
                        isNormalUser = true;
                        extraGroups = [ "wheel" ];
                        uid = 1000;
                      };

                      system.stateVersion = "22.05";

                      # virtualisation.graphics = false;

                      environment.systemPackages = [
                        self.packages.${system}.default
                      ];

                      systemd.user.services = {
                        "org.gnome.Shell@wayland" = {
                          serviceConfig = {
                            ExecStart = [
                              ""
                              "${pkgs.gnome.gnome-shell}/bin/gnome-shell"
                            ];
                          };
                        };
                      };
                    };
                  };

                  testScript = { nodes, ... }:
                    let
                      lib = pkgs.lib;
                      l = lib.lists;

                      user = nodes.machine.users.users.alice;
                      username = user.name;

                      type = word: "machine.send_chars(\"${word}\")";
                      key = key: "machine.send_key(\"${key}\")";
                      sleep = duration: "machine.sleep(${toString duration})";

                      execution = [
                        (type "Berlin Hbf")
                        (sleep 2)
                        (key "tab")
                        (type "PARIS")
                        (sleep 2)
                        (l.replicate 5 (key "tab"))
                        (key "ret")
                        (sleep 5)
                        (l.replicate 11 (key "tab"))
                        (key "ret")
                        (l.replicate 3 (key "tab"))
                        (key "ret")
                      ];


                      preExecution = [
                        (sleep 20)
                        (type "Railway")
                        (key "ret")
                      ];

                      postExecution = [
                        (key "alt-print") # XXX: This for some reason sometimes fails. No idea why.
                        "machine.execute(\"mv /home/${username}/Pictures/Screenshots/* screenshot.png\")"
                        "machine.copy_from_vm(\"screenshot.png\", \".\")"
                      ];

                      fullExecution = l.flatten [preExecution (sleep 5) execution (sleep 5) postExecution];

                      code = lib.concatStringsSep "\nmachine.sleep(1)\n" fullExecution;
                    in
                      code;
                }
              ];
            };
        })
    );
}
