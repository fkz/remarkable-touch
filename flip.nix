{ stdenv }:

stdenv.mkDerivation {
  pname = "touch-page";
  version = "0.1.0";

  src = ./flip;

  buildPhase = ''
    $CC main.c -o $out  
  '';
}