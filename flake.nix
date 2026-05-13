{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    claude-code-nix.url = "github:sadjow/claude-code-nix";
  };

  outputs = { nixpkgs, claude-code-nix, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = [
          pkgs.cargo
          pkgs.rustc
          pkgs.rustfmt

          claude-code-nix.packages.${system}.claude-code # dev experience
        ];
      };
    };
}
