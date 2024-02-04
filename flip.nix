{ stdenv, rustPlatform }:

stdenv.mkDerivation {
  pname = "touch-page";
  version = "0.1.0";

  src = ./flip;

  buildPhase = ''
    $CC main.c -o $out
  '';

  fixupPhase = ''
    patchelf --set-interpreter /lib/ld-linux-armhf.so.3 $out
  '';
}

# rustPlatform.buildRustPackage {
#   pname = "flip";
#   version = "0.1.0";

#   src = ./flip;

#   cargoHash = "sha256-nnhd7Qfz08tE4Xv1INLaYdMzJjekX1YImr+3bKpnhAg=";

#   bin = "bin/flip";

#   fixupPhase = ''
#     patchelf --set-interpreter /lib/ld-linux-armhf.so.3 $out/$bin
#   '';
# }