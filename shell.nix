let
  sources = import ./nix/sources.nix;
  rust = import ./nix/rust.nix { inherit sources; };
  pkgs = import sources.nixpkgs { };
  niv = import sources.niv { inherit sources; };
in pkgs.mkShell { buildInputs = [ rust pkgs.cargo-fuzz niv.niv ]; }
