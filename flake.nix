{
  description = "Tools for the Remarkable";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/release-23.11";

  outputs = { self, nixpkgs }: let pkgs = nixpkgs.legacyPackages.x86_64-linux; in {

    packages.x86_64-linux = rec {
      flip = pkgs.callPackage ./flip.nix { inherit toolchain; };
      toolchain = pkgs.callPackage ./toolchain.nix {};
      installer = pkgs.callPackage ./installer.nix { inherit flip; };
      default = installer;
    };
  };
}
