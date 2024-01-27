{ stdenv, fetchurl, python3, which, writeScript }:

let toolchain = 
    stdenv.mkDerivation (self: {
      pname = "cortex-toolchain";
      version = "4.0.117";

      src = fetchurl {
        url = "https://storage.googleapis.com/remarkable-codex-toolchain/remarkable-platform-image-${self.version}-rm2-public-x86_64-toolchain.sh";
        hash = "sha256-WfJKWAZznJJI2/WQTwQCh/GMvPUcj783HIA301uuYmg=";
        executable = true;
      };

      nativeBuildInputs = [ python3 which ];

      dontUnpack = true;

      buildPhase = ''
        $src -d $out
      '';
    }); in
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