{ writeScriptBin, nodejs, programs }:

writeScriptBin "install-remarkable-tools" ''
  #!${nodejs}/bin/node
  const tools = { ${builtins.concatStringsSep ", " (map (name: ''"${name}": "${programs.${name}}"'') (builtins.attrNames programs)) } };
  require("${./installer/install.js}")(tools);
''
