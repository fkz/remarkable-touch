{ stdenv, fetchFromGitHub, zlib, avahi, pkg-config, gnutls, which, nix-bundle, lib, perlPackages, cups }:

let 
  avahiOverride = (avahi.override { inherit perlPackages; }).overrideAttrs {
    preConfigure = ''
      sed -i 's/if test x"$acx_pthread_ok" = xyes; then/if test x"$acx_pthread_ok" = xno; then/' configure 
    '';

    makeFlags = [ "AM_LDADD=-zmuldefs" ];

   };
  cupsOverride = ((cups.override) { avahi = avahiOverride; }).overrideAttrs (prev: {
    configureFlags = let x = prev.configureFlags ++ [ "--with-components=libcups" ]; in builtins.trace x x;
  });

ippsample = stdenv.mkDerivation (self: with self; {
  pname = "ippsample";
  version = "2023.09";

  nativeBuildInputs = [ pkg-config which ];


  buildInputs = [ zlib gnutls cupsOverride avahiOverride ]; 
  #(avahi.overrideAttrs (self: {
  #  propagatedBuildInputs = builtins.filter (x: !(lib.strings.hasPrefix "perl" x.name )) self.propagatedBuildInputs;
  #}))];
  
  src = fetchFromGitHub {
    owner = "istopwg";
    repo = pname;
    rev = "826f046380f1085b46a0704867934d1a6b06404b";
    sha256 = "sha256-BERZosx0i5NlIJRMbKyAFd8r+SOXzJA/gnYH5ZGMPpc=";
    fetchSubmodules = true;
  };

  # ar needs to be an absolute path
  preConfigure = ''
    export AR=$(which $AR)
    sed -i s/pkg-config/$PKG_CONFIG/ Makedefs.in
  '';

  #configureFlags = [ "--help" ];

  makeFlags = [ "IPPTOOLS=ippfind" ];

  # do a 
  # postBuild = ''
  #   cd server
  #   rm ippserver
  #   echo "Try installing"
  #   $CC -Wl,-static-g -Os -o ippserver auth.o client.o conf.o device.o ipp.o job.o log.o main.o printer.o resource.o subscription.o transform.o -L../libcups/cups -lcups3  -ldl -lm
  #   cd ..
  # '';
}); in ippsample

# stdenv.mkDerivation {
#   name = "ippserver";

#   dontUnpack = true;
#   buildPhase = "cp ${ippsample}/bin/ippserver $out";
# }

# stdenv.mkDerivation {
#   name = "ippserver";

#   dontUnpack = true;

#   nativeBuildInputs = [ nix-bundle ];

#   buildPhase = ''
#     nix-bundle $out ${ippsample}/bin/ippserver
#   '';
# }