{ sources ? import ./sources.nix }:

let
  pkgs =
    import sources.nixpkgs { overlays = [ (import sources.nixpkgs-mozilla) ]; };
  channel = "stable";
  version = "1.43.0";
  targets = [ ];
  extensions = [ "rust-src" "rls-preview" "rust-analysis" "rustfmt-preview" ];
  rustChannelOfTargetsAndExtensions = channel: version: targets: extensions:
    (pkgs.rustChannelOf { inherit channel version; }).rust.override {
      inherit targets extensions;
    };
  chan = rustChannelOfTargetsAndExtensions channel version targets extensions;
in chan
