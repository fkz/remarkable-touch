{ rustPlatform }:

rustPlatform.buildRustPackage {
  pname = "ipp-server";
  version = "0.1.0";

  src = ./ipp-server;

  cargoHash = "sha256-pXhWdjVaPQidDsSjI4NW3STaozAwBjf7haH/Zk7pHC0=";

  bin = "bin/remarkable-ipp";

  fixupPhase = ''
    patchelf --set-interpreter /lib/ld-linux-armhf.so.3 $out/$bin
  '';
}