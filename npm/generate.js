import * as fs from "fs/promises";
import { resolve, dirname } from "path";

const __dirname = dirname(new URL(import.meta.url).pathname);

function getArguments() {
    const args = {};
    for (let i = 2; i < process.argv.length; i += 2) {
        const key = process.argv[i].replace('--', '');
        const value = process.argv[i + 1];
        args[key] = value;
    }
    return args;
}

const { target, build, version } = getArguments();

if (!target || !build || !version) {
    console.error('Usage: node <script.js> --target <target> --build <build> --version <version>');
    process.exit(1);
}

const targetRegex = /^([a-zA-Z0-9_]+)-([a-zA-Z0-9_]+)-([a-zA-Z0-9_\-]+)/;
const match = target.match(targetRegex);

if (!match) {
    console.error('Invalid target format. Expected format: <cpu>-<vendor>-<os>');
    process.exit(1);
}

const [, processor] = build.split('-');

const [, cpu, flavour, os] = match;

async function genPlatformPackage() {
    let name;
    if (flavour) {
        name = `${cpu}-${flavour}-${os}`;
    } else {
        name = `${cpu}-${os}`;
    }
    const platformPackage = {
        name: `@tailcallhq/${build}`,
        version,
        description: `Tailcall ${build} Platform`,
        // Modify the `bin` property here to be an object instead of a `directories` property
        bin: {
            "tc": "./bin/tailcall" // Command 'tc' points to the executable 'tailcall'
        },
        os: [os],
        cpu: [processor],
    };

    const filePath = resolve(__dirname, `@tailcallhq/${build}/bin`);
    await fs.mkdir(filePath, { recursive: true });
    await fs.writeFile(
        resolve(filePath, "../package.json"),
        JSON.stringify(platformPackage, null, 2),
        "utf8"
    );

    // Copy the executable to the bin directory as before
    await fs.copyFile(
        resolve(__dirname, "../target", name, "release/tailcall"),
        resolve(filePath, "tailcall")
    );
}

await genPlatformPackage();
