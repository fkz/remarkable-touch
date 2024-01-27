{ stdenv, fetchurl, bash, python3, which, writeScript }:

let toolchain = 
    stdenv.mkDerivation (self: {
      pname = "cortex-toolchain";
      version = "4.0.117";

      src = fetchurl {
        url = "https://storage.googleapis.com/remarkable-codex-toolchain/remarkable-platform-image-${self.version}-rm2-public-x86_64-toolchain.sh";
        #sha256 = "29779c80db2a025126d52faad88d553cadda09fff31fb4138a9df1d5b7e8a247";
        hash = "sha256-WfJKWAZznJJI2/WQTwQCh/GMvPUcj783HIA301uuYmg=";
        executable = true;
      };

      nativeBuildInputs = [ bash python3 which ];

      dontUnpack = true;

      buildPhase = ''
        $src -d $out
      '';
    }); in
derivation {
  name = "touch-page-0.1.0";

  system = "x86_64-linux";
  
  builder = writeScript "builder" ''
    #!${bash}/bin/bash

    source ${toolchain}/environment-setup-cortexa7hf-neon-remarkable-linux-gnueabi
    $CC $src/main.c -o $out
  '';

  src = ./src;

}