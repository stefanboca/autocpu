{
  lib,
  pkgs,
  ...
}: let
  inherit (lib.meta) getExe;
in {
  projectRootFile = "flake.nix";

  settings.formatter.tombi = {
    command = "${getExe pkgs.tombi}";
    options = ["format" "--offline"];
    includes = ["*.toml"];
  };

  programs = {
    # keep-sorted start
    alejandra.enable = true;
    deadnix.enable = true;
    keep-sorted.enable = true;
    rustfmt.enable = true;
    # keep-sorted end
  };
}
