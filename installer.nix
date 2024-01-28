{ writeScriptBin, nodejs, flip }:

writeScriptBin "install-remarkable-tools" ''
  #!${nodejs}/bin/node
  require("${./installer/install.js}")
''
