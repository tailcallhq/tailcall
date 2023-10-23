import { copyFile, mkdir, writeFile } from 'fs/promises';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const basePkg = {
  "description": "",
  "scripts": {
    "test": "echo \"Error: no test specified\" && exit 1"
  },
  "keywords": [
    "tailcall",
    "graphql",
    "graphiql",
    "rust"
  ],
  "author": "Taillcall contributors",
  "license": "Apache-2.0",
  "repository": {
    "type": "git",
    "url": "https://github.com/tailcall/tailcall.git"
  },
  "bugs": {
    "url": "https://github.com/tailcall/tailcall/issues"
  },
  "homepage": "https://github.com/tailcall/tailcall#readme",
};

const { binary_source, node_os, node_arch, node_package, version } = process.env;

const prepare = async () => {
  const __dirname = dirname(fileURLToPath(import.meta.url));
  const destination = join(__dirname, 'packages', node_package);

  await mkdir(destination, { recursive: true });

  const packageJson = {
    name: node_package,
    version,
    ...basePkg,
    os: [node_os],
    cpu: [node_arch],
  }

  await copyFile(
    join(__dirname, '..', binary_source),
    join(destination, 'bin')
  )

  await writeFile(
    join(destination, 'package.json'),
    JSON.stringify(packageJson, null, 2),
  );
}

prepare();
