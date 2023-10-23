import { access, constants, copyFile, readFile, rm, writeFile } from 'fs/promises';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';
import { parse } from 'toml';
import templatePkg from '../package.template.json' assert { type: 'json' }

const __dirname = dirname(fileURLToPath(import.meta.url));
const BASE_DIR = join(__dirname, '..', '..');

/**
 * @param {String} destination
 * @param {String} binarySource
 */
export const copyBin = (destination, binarySource) => copyFile(
  join(BASE_DIR, binarySource),
  join(destination, 'bin')
);

/**
 * @param {String} destination
 * @param {Record<string, any>} pkgData 
 */
export const writePackageJson = async (destination, { name, version, ...pkgData }) => {
  const cargoToml = await readFile(join(BASE_DIR, 'Cargo.toml'));

  const { license, description } = parse(cargoToml.toString());

  const pkgPath = join(BASE_DIR, 'npm', destination, 'package.json');
  const basePkg = {
    name,
    version,
    license,
    description,
  };

  try {
    // this method will throw an error if file does not exists
    await access(pkgPath, constants.F_OK)

    Object.assign(basePkg, JSON.parse(await readFile(pkgPath)));
    await rm(pkgPath);
  } catch (error) {}

  const packageJson = {
    ...basePkg,
    ...templatePkg,
    ...pkgData,
  };

  await writeFile(pkgPath, JSON.stringify(packageJson, null, 2));
}
