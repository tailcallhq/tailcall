import { copyFile } from 'fs/promises';
import { writePackageJson } from './lib/index.js';

const { version } = process.env;

const optionalDependencies = {
  "@tailcallhq/core-linux-x86_64-gnu": "*",
  "@tailcallhq/core-linux-x86_64-musl": "*",
  "@tailcallhq/core-linux-aarch64-gnu": "*",
  "@tailcallhq/core-linux-aarch64-musl": "*",
  "@tailcallhq/core-linux-i686-gnu": "*",
  "@tailcallhq/core-darwin-arm64": "*",
  "@tailcallhq/core-darwin-x86_64": "*"
}

await writePackageJson('.', {
  version,
  optionalDependencies,
});

await copyFile('../README.md', './README.md')

console.log('aaaaa')
