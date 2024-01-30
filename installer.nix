{ writeScriptBin, nodejs, flip, web-interface, printer, ippserver }:

writeScriptBin "install-remarkable-tools" ''
  #!${nodejs}/bin/node
  const tools = {
    flip: "${flip}",
    "web-interface": "${web-interface}",
    printer: "${printer}/bin/${printer.pname}",
    ippserver: "${ippserver}"
  };
  require("${./installer/install.js}")(tools);
''
