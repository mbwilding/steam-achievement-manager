{
  description = "Steam Achievement Manager CLI";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      packages = forAllSystems (pkgs: rec {
        steam-achievement-manager = pkgs.rustPlatform.buildRustPackage {
          pname = "steam-achievement-manager";
          version = (pkgs.lib.importTOML ./Cargo.toml).package.version;
          src = self;
          cargoLock.lockFile = ./Cargo.lock;
          doCheck = false;
          nativeBuildInputs = [ pkgs.perl ];
          postInstall = ''
            lib=$(find target -name 'libsteam_api.*' -path '*/build/steamworks-sys-*/out/*' | head -n1)
            install -Dm755 "$lib" "$out/bin/$(basename "$lib")"
          '';
          meta = {
            description = "Steam Achievement Manager CLI";
            homepage = "https://github.com/mbwilding/steam-achievement-manager";
            license = pkgs.lib.licenses.mit;
            mainProgram = "sam";
          };
        };
        default = steam-achievement-manager;
      });

      overlays.default = final: _prev: {
        steam-achievement-manager = self.packages.${final.stdenv.hostPlatform.system}.steam-achievement-manager;
      };

      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            rustc
            clippy
            rustfmt
            rust-analyzer
            perl
          ];
        };
      });
    };
}
