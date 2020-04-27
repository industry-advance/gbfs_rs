let
  sources = import ./nix/sources.nix;
  rust = import ./nix/rust.nix { inherit sources; };
  nixpkgs = import sources.nixpkgs { };
in nixpkgs.mkShell { buildInputs = [ rust ]; }
