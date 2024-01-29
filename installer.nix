{ writeScriptBin, nodejs, flip, web-interface, printer }:

writeScriptBin "install-remarkable-tools" ''
  #!${nodejs}/bin/node
  const tools = {
    flip: "${flip}",
    "web-interface": "${web-interface}",
    printer: "${printer}/bin/${printer.pname}"
  };
  require("${./installer/install.js}")(tools);
''
