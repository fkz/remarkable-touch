{ writeScriptBin, nodejs, flip, web-interface }:

writeScriptBin "install-remarkable-tools" ''
  #!${nodejs}/bin/node
  const tools = {
    flip: "${flip}",
    "web-interface": "${web-interface}"
  };
  require("${./installer/install.js}")(tools);
''
