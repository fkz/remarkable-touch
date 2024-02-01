{ rustPlatform }:

rustPlatform.buildRustPackage {
  pname = "ipp-server";
  version = "0.1.0";

  src = ./ipp-server;

  cargoHash = "sha256-d+QfKJaxZh62GnvBtXp+a7p+urW18DU37f7qLWkTeIY=";

  bin = "bin/remarkable-ipp";

  fixupPhase = ''
    patchelf --set-interpreter /lib/ld-linux-armhf.so.3 $out/$bin
  '';
}