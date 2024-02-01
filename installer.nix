{ writeScriptBin, nodejs, programs }:

let executable = program: if program ? bin then "${program}/${program.bin}" else "${program}"; in

writeScriptBin "install-remarkable-tools" ''
  #!${nodejs}/bin/node
  const tools = { ${builtins.concatStringsSep ", " (map (name: ''"${name}": "${executable(programs.${name})}"'') (builtins.attrNames programs)) } };
  require("${./installer/install.js}")(tools);
''
