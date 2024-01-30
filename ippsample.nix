{ stdenv, fetchFromGitHub, zlib, avahi }:

stdenv.mkDerivation (self: with self; {
  pname = "ippsample";
  version = "2023.09";

  buildInputs = [ zlib avahi ];

  src = fetchFromGitHub {
    owner = "istopwg";
    repo = pname;
    rev = "refs/tags/v${version}";
    sha256 = "sha256-rzY5GmnfzsStHWL1wUUMVhfHyAX9v59AL/2CXKvUExw=";
    fetchSubmodules = true;
  };
})