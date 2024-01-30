{ buildGoModule, fetchFromGitHub }:

buildGoModule rec {
  pname = "remarkable_printer";
  version = "11.0.2";

  src = fetchFromGitHub {
    owner = "Evidlo";
    repo = pname;
    rev = "refs/tags/v${version}";
    sha256 = "sha256-NBq6bK/uVrL7KhM4MxAmvkfvRf8dB2Mzvn0oRAm/nTE=";
  };

  vendorHash = "sha256-PfJNr7t/27PSnwIwFv0kHV3f+er0fpHwqddS8yS7ofo=";

  fixupPhase = ''
    patchelf --set-interpreter /lib/ld-linux-armhf.so.3 $out/bin/${pname}
  '';
}