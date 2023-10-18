const { spawnSync } = require('child_process');

/**
 * @returns {String}
 */
function getExecutionPath() {
    const arch = process.arch;
    let os = process.platform;

    if (os === 'darwin') {
        // our builds use apple instead of darwin
        os = 'apple'
    }

    try {
        const absolutePath = require.resolve(`tailcall-${os}-${arch}/bin/tailcall`)
        return absolutePath
    } catch (e) {
        throw new Error(`Couldn't find tailcall binary inside node_modules for ${os}-${arch}`)
    }
}

function run(argv) {
    // the two first arguments are the node bin and the package bing
    const args = argv.slice(2)

    const { status } = spawnSync(
        getExecutionPath(),
        args,
        { stdio: 'inherit' }
    );

    process.exit(status ?? 0)
}

module.exports = { run }