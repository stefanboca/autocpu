{
  inputs = {
    nixpkgs.url = "https://channels.nixos.org/nixos-unstable/nixexprs.tar.xz";

    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
    treefmt-nix,
    ...
  }: let
    inherit (nixpkgs) lib;
    inherit (lib.attrsets) genAttrs mapAttrs' nameValuePair;
    inherit (lib.fileset) fileFilter toSource unions;
    inherit (lib.modules) importApply;
    inherit (lib.trivial) importTOML;

    systems = ["x86_64-linux" "aarch64-linux"];
    forAllSystems = f: genAttrs systems (system: f (import nixpkgs {inherit system;}));

    cargoToml = importTOML ./Cargo.toml;

    mkPackages = pkgs: {
      autocpu = pkgs.rustPlatform.buildRustPackage {
        pname = cargoToml.package.name;
        inherit (cargoToml.package) version;

        src = toSource {
          root = ./.;
          fileset = unions [
            ./Cargo.lock
            ./Cargo.toml
            ./res/autocpu.service.in
            ./res/org.stefanboca.AutoCpu.service.in
            ./res/org.stefanboca.AutoCpu.conf
            (fileFilter (file: file.hasExt "rs") ./.)
          ];
        };

        cargoLock = {
          lockFile = ./Cargo.lock;
          allowBuiltinFetchGit = true;
        };

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
      };
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

    devShells = forAllSystems (pkgs: {
      default = pkgs.mkShellNoCC {};
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
