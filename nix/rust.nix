{ sources ? import ./sources.nix }:

let
  pkgs =
    import sources.nixpkgs { overlays = [ (import sources.nixpkgs-mozilla) ]; };
  channel = "nightly";
  version = "2021-07-07";
  targets = [];
  extensions = [ "rust-src" "clippy-preview" "rust-analyzer-preview" ];
  rustChannelOfTargetsAndExtensions = channel: version: targets: extensions:
    (pkgs.rustChannelOf { inherit channel version; }).rust.override {
      inherit targets extensions;
    };
  chan = rustChannelOfTargetsAndExtensions channel version targets extensions;
in
chan
