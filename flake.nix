{
  description = "Tools for the Remarkable";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-23.11";
    flake-utils.url = "github:numtide/flake-utils";
    bundlers.url = "github:NixOS/bundlers";
  };

  outputs = { self, nixpkgs, flake-utils, bundlers }: 
    flake-utils.lib.eachDefaultSystem (system: 
      let pkgs = nixpkgs.legacyPackages.${system};
          crossPkgs = import nixpkgs.outPath {
            inherit system;
            crossSystem = nixpkgs.lib.systems.examples.remarkable2;
          }; in {
      packages = rec {
        flip = crossPkgs.callPackage ./flip.nix {};
        web-interface = crossPkgs.callPackage ./web-interface.nix {};
        printer = crossPkgs.callPackage ./printer.nix {};
        ipp-server = crossPkgs.callPackage ./ipp-server.nix {};
        
        installer = pkgs.callPackage ./installer.nix { inherit flip web-interface printer ipp-server; };
        default = installer;

        toolchain = pkgs.callPackage ./toolchain.nix {};
      };
  });
}
