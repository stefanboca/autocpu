self: {
  config,
  lib,
  pkgs,
  ...
}: let
  inherit (lib.options) mkEnableOption mkOption mkPackageOption;
  inherit (lib.modules) mkIf;

  toml = pkgs.formats.toml {};

  configFile = toml.generate "autocpu.config" cfg.settings;

  cfg = config.services.autocpu;
in {
  options.services.autocpu = {
    enable = mkEnableOption "autocpu";

    package = mkPackageOption pkgs "autocpu" {default = self.packages.${pkgs.stdenv.hostPlatform.system}.autocpu;};

    settings = mkOption {
      inherit (toml) type;
      default = {};
    };
  };

  config = mkIf cfg.enable {
    environment.systemPackages = [cfg.package];

    services.dbus.packages = [cfg.package];
    services.upower.enable = true;

    systemd = {
      packages = [cfg.package];
      services.autocpu = {
        environment.AUTOCPU_CONFIG = configFile;
        # Workaround for https://github.com/NixOS/nixpkgs/issues/81138
        wantedBy = ["multi-user.target"];
      };
    };
  };
}
