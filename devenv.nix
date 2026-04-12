{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:
let
  name = "kuberaid";
in
{
  cachix.enable = true;
  cachix.pull = [ "m00nwtchr" ];

  # https://devenv.sh/packages/
  packages = with pkgs; [
    git
    cargo-nextest
    cargo-audit
    just
    gum
    talosctl
    openssl
    cargo-watch
  ];

  # https://devenv.sh/languages/
  languages.rust = {
    enable = true;
    mold.enable = true;
  };

  treefmt = {
    enable = true;
    config.programs = {
      nixfmt.enable = true;
      rustfmt.enable = true;
      taplo.enable = true;
    };
  };

  # https://devenv.sh/git-hooks/
  git-hooks.hooks = {
    treefmt.enable = true;
    clippy.enable = true;
  };

  # See full reference at https://devenv.sh/reference/options/
}
