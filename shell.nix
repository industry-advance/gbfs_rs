let
  sources = import ./nix/sources.nix;
  rust = import ./nix/rust.nix { inherit sources; };
  nixpkgs = import sources.nixpkgs { };
  niv = import sources.niv { inherit sources; };
in nixpkgs.mkShell { buildInputs = [ rust niv.niv ]; }
