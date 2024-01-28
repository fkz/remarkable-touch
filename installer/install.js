const path = require('node:path');

const spawnWithoutPromise = require('node:child_process').spawn;

let remarkableHost;
if (process.env["REMARKABLE_HOST"]) {
  remarkableHost = "root@" + process.env["REMARKABLE_HOST"];
}


const appDir = "ourAdditionalPrograms"

function spawn(...args) {
  return new Promise((resolve, reject) => {
    const p = spawnWithoutPromise(...args);
    p.on("error", err => reject(err));
    p.on("close", exitCode => {
      if (exitCode === 0) {
        resolve(p);
      } else {
        reject(exitCode)
      }
    })
  });
}


async function copyFile(sourcePath, destDir, name) {
  await spawn("ssh", [remarkableHost, "mkdir", "-p", destDir], { stdio: "inherit" });
  const targetPath = path.join(destDir, name);
  await spawn("scp", [sourcePath, remarkableHost + ":" + targetPath ], { stdio: "inherit"});
}

async function installProgram(name, path) {
  console.log("Copying program to device: " + name);
  await copyFile(path, appDir, name);
  console.log("Succesfully installed program: " + name);
}

async function executeProgram(name) {
  const programPath = path.join(appDir, name);
  await spawn("ssh", ["-t", remarkableHost, programPath], { stdio: "inherit" });
}

module.exports = async (programs) => {
  const command = process.argv[2];
  const args = process.argv.slice(3);

  if (command === "help" || command === undefined) {
    console.log(`
      install-remarkable-tools can be used to install tools to the Remarkable2.

      Set REMARKABLE_HOST to the IP address of your Remarkable2. This tool uses ssh
      to copy files to the remarkable and execute them.

      Usage: remarkable-tools list
      List tools, that can be installed

      Usage: remarkable-tools install [tools]
      Install tools to remarkable. tools is a space-seperated list of the tools.

      Usage remarkable-tools run [tools]
      Run the specified tools
`);
    return;
  }

  if (command === "list") {
    console.log("The following tools are available:");
    for (const name in programs) {
      console.log("   " + name);
    }
    return;
  }

  if (command === "install") {
    if (!remarkableHost) {
      console.log("You need to specify REMARKABLE_HOST env var to execute this program");
      process.exitCode = 2;
      return;
    }

    if (args.length === 0) {
      console.log("You need to specifiy at least one program to install");
      console.log("To install all programs, use \"all\"");
      process.exitCode = 1;
      return;
    }

    let names = args;

    if (args.length === 1 && args[0] === "all") {
      names = Object.keys(programs);
    }

    const invalidNames = names.filter(name => !programs[name]);
    if (invalidNames.length > 0) {
      console.log("Invalid program names: ", invalidNames.join(", "));
      process.exitCode = 1;
      return;
    }

    try {
      for (const name of names) {
        await installProgram(name, programs[name]);
      }
    } catch (e) {
      console.error("Some error occured", e);
      process.exitCode = 1;
    }
    return;
  }

  if (command === "run") {
    if (args.length != 1) {
      console.log("Expected program name as argument.");
      process.exitCode = 1;
      return;
    }

    if (!programs[args[0]]) {
      console.log("Unknown program: ", args[0]);
      process.exitCode = 1;
      return;
    }

    await executeProgram(args[0]);
    return;
  }

  console.log("Unknown command: ", command);
  process.exitCode = 1;
}