{
  description = "Tools for the Remarkable";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-23.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }: 
    flake-utils.lib.eachDefaultSystem (system: 
      let pkgs = nixpkgs.legacyPackages.${system};
          crossPkgs = import nixpkgs.outPath {
            inherit system;
            crossSystem = nixpkgs.lib.systems.examples.remarkable2;
          }; in {
      packages = rec {
        flip = crossPkgs.callPackage ./flip.nix {};
        web-interface = crossPkgs.callPackage ./web-interface.nix {};
        toolchain = pkgs.callPackage ./toolchain.nix {};
        installer = pkgs.callPackage ./installer.nix { inherit flip web-interface; };
        printer = crossPkgs.callPackage ./printer.nix {};
        default = installer;
      };
  });
}
