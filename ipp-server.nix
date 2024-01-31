{ stdenv }:

stdenv.mkDerivation {
  pname = "ipp-server";
  version = "0.1.0";

  src = ./ipp-server;

  buildInputs = [ ];

  buildPhase = ''
    $CC main.c -o $out
  '';

  fixupPhase = ''
    patchelf --set-interpreter /lib/ld-linux-armhf.so.3 $out
  '';
}