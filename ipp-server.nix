{ rustPlatform }:

rustPlatform.buildRustPackage {
  pname = "ipp-server";
  version = "0.1.0";

  src = ./ipp-server;

  cargoHash = "sha256-10YsNA31NzcfKkPhAt3jhmhsKuDbHoyriFZhMD5wKHE=";

  bin = "bin/remarkable-ipp";

  fixupPhase = ''
    patchelf --set-interpreter /lib/ld-linux-armhf.so.3 $out/$bin
  '';
}