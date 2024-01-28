{ stdenv, fetchurl, python3, which, writeScript }:

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
})