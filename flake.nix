{
  description = "A basic flake with a shell";

  # Inputs
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.systems.url = "github:nix-systems/default";
  inputs.flake-utils = {
    url = "github:numtide/flake-utils";
    inputs.systems.follows = "systems";
  };    
  inputs.fenix = {
    url = "github:nix-community/fenix";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  # Outputs
  outputs = { nixpkgs, flake-utils, fenix, ... }: 
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in {
        devShells.default = pkgs.mkShell { 
          packages = with pkgs; [
            bashInteractive
            bacon 
            # rust-analyzer
            just
            clippy
            rustup
            clang
            llvmPackages.bintools
            pkg-config
            openssl
          ];  
        };
      }
    );
}
