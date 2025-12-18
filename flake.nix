{
  inputs = {
    nixpkgs.url = "https://channels.nixos.org/nixos-unstable/nixexprs.tar.xz";
    crane.url = "github:ipetkov/crane";

    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    treefmt-nix,
    ...
  }: let
    inherit (nixpkgs) lib;
    inherit (lib.attrsets) genAttrs mapAttrs' nameValuePair;
    inherit (lib.fileset) toSource unions;
    inherit (lib.modules) importApply;

    systems = ["x86_64-linux" "aarch64-linux"];
    forAllSystems = f: genAttrs systems (system: f (import nixpkgs {inherit system;}));

    mkPackages = pkgs: let
      craneLib = crane.mkLib pkgs;

      commonArgs = {
        src = toSource {
          root = ./.;
          fileset = unions [
            (craneLib.fileset.commonCargoSources ./.)
            ./res
          ];
        };
        strictDeps = true;
      };

      cargoArtifacts = craneLib.buildDepsOnly commonArgs;
    in {
      autocpu = craneLib.buildPackage (commonArgs
        // {
          inherit cargoArtifacts;

          postInstall = ''
            mkdir -p $out/lib/systemd/system
            substitute  ./res/autocpu.service.in $out/lib/systemd/system/autocpu.service --subst-var out

            mkdir -p $out/share/dbus-1/{system.d,system-services}
            mv ./res/org.stefanboca.AutoCpu.conf $out/share/dbus-1/system.d/
            substitute ./res/org.stefanboca.AutoCpu.service.in $out/share/dbus-1/system-services/org.stefanboca.AutoCpu.service --subst-var out
          '';

          meta = {
            mainProgram = "autocpu";
          };
        });
    };

    treefmt = forAllSystems (pkgs: treefmt-nix.lib.evalModule pkgs ./treefmt.nix);
  in {
    packages = forAllSystems (pkgs: let
      packages = mkPackages pkgs;
    in
      packages // {default = packages.autocpu;});

    overlays = {
      default = _final: mkPackages;
    };

    nixosModules.default = importApply ./nix/nixos.nix self;

    devShells = forAllSystems (pkgs: let
      craneLib = crane.mkLib pkgs;
      packages = mkPackages pkgs;
    in {
      default = craneLib.devShell {
        inputsFrom = [packages.autocpu];
      };
    });

    formatter = forAllSystems (pkgs: treefmt.${pkgs.stdenv.hostPlatform.system}.config.build.wrapper);

    checks = forAllSystems (pkgs: let
      inherit (pkgs.stdenv.hostPlatform) system;

      packages = mapAttrs' (n: nameValuePair "package-${n}") self.packages.${system};
      devShells = mapAttrs' (n: nameValuePair "devShell-${n}") self.devShells.${system};
      formatting = {formatting = treefmt.${system}.config.build.check self;};
    in
      packages // devShells // formatting);
  };
}
