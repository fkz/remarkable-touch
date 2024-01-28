{ stdenv, writeScript, toolchain }:

derivation {
  name = "touch-page-0.1.0";

  system = "x86_64-linux";
  
  builder = writeScript "builder" ''
    #!${stdenv.shell}

    source ${toolchain}/environment-setup-cortexa7hf-neon-remarkable-linux-gnueabi
    $CC $src/main.c -o $out
  '';

  src = ./src;
}