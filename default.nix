{ 
  nixpkgs ? import <nixpkgs> {}, 
  stdenv ? nixpkgs.stdenv, 
  fetchurl ? nixpkgs.fetchurl, 
  bash ? nixpkgs.bash, 
  python3 ? nixpkgs.python3, 
  which ? nixpkgs.which 
}:

let toolchain = 
    stdenv.mkDerivation {
      pname = "cortex-toolchain";
      version = "3.1.15";

      src = fetchurl {
        url = "https://storage.googleapis.com/remarkable-codex-toolchain/codex-x86_64-cortexa7hf-neon-rm11x-toolchain-3.1.15.sh";
        sha256 = "07s1cqz1knj0kvbxa67fmzl13p7dl3dipn8ai67pkdpbkd7mp6hs";
      };

      nativeBuildInputs = [ bash python3 which ];

      dontUnpack = true;

      buildPhase = ''
        bash $src -d $out
      '';
    }; in
stdenv.mkDerivation {
  name = "test-program";

  src = ./src;

  buildPhase = ''
    source ${toolchain}/environment-setup-cortexa7hf-neon-remarkable-linux-gnueabi

    $CC main.c -o $out
    chmod +x $out
  '';
}