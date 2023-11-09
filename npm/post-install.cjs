var os = require('os');
var fs = require('fs');

// Define the valid platform builds
var validPlatformBuilds = ['win32', 'darwin', 'linux'];

// Get the current platform
var platform = os.platform();

if (validPlatformBuilds.includes(platform)) {
  // Check if the platform build exists
  var buildExists = fs.existsSync(`./builds/${platform}`);
  
  if (!buildExists) {
    throw new Error(`No platform build found for ${platform}. Please create an issue on our GitHub repository.`);
  }
} else {
  throw new Error(`Platform ${platform} is not supported. Please create an issue on our GitHub repository 'https://github.com/tailcallhq/tailcall/issues/new/choose'.`);
}
console.log("taicall added succesfully");