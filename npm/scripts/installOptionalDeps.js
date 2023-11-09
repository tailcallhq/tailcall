// tailcall-npm-generator/scripts/installOptionalDeps.js

const exec = require('child_process').exec;
const optionalDependencies = require('../@tailcallhq/tailcall/scripts/installOptionalDeps.js');

Object.entries(optionalDependencies).forEach(([pkg, version]) => {
  exec(`npm install ${pkg}@${version} --no-save`, (error, stdout, stderr) => {
    if (error) {
      console.error(`Failed to install optional dependency: ${pkg}`, stderr);
    } else {
      console.log(`Successfully installed optional dependency: ${pkg}`, stdout);
    }
  });
});
