import { spawnSync } from 'child_process';

const APP_VERSION = process.env.APP_VERSION

spawnSync('npm', ['version', APP_VERSION.replace('v', '')]);
spawnSync('npm', ['publish']);
