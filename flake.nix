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
        ippserver = let replaceHead = drv:
          pkgs.runCommand "${drv.name}" { nativeBuildInputs = [ pkgs.gnused ]; srcFile = "${drv}"; } ''
            sed 's/head -c /dd bs=1 count=/g' $srcFile > $out
            chmod +x $out
          ''; in #replaceHead(bundlers.defaultBundler.${system} 
          (crossPkgs.pkgsStatic.callPackage ./ippserver.nix {});
        
        installer = pkgs.callPackage ./installer.nix { inherit flip web-interface printer ippserver; };
        default = installer;

        toolchain = pkgs.callPackage ./toolchain.nix {};
      };
  });
}
