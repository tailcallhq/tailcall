import { mkdir } from 'fs/promises';
import { join } from 'path';
import { copyBin, writePackageJson } from './lib/index.js';

const { binary_source, node_os, node_arch, node_package, version } = process.env;

const prepare = async () => {
  const destination = join('packages', node_package);

  await mkdir(destination, { recursive: true });

  await copyBin(destination, binary_source)

  await writePackageJson(destination, {
    name: node_package,
    version,
    os: [node_os],
    cpu: [node_arch],
  });
}

prepare();
