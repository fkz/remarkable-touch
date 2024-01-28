{ writeScriptBin, nodejs, flip }:

writeScriptBin "install-remarkable-tools" ''
  #!${nodejs}/bin/node
  const tools = {
    flip: "${flip}"
  };
  require("${./installer/install.js}")(tools);
''
