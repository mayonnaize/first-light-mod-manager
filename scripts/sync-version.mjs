// package.json のバージョンを tauri.conf.json と Cargo.toml に同期
import { readFileSync, writeFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const { version } = JSON.parse(readFileSync(resolve(root, 'package.json'), 'utf8'));

// tauri.conf.json
const tauriConf = resolve(root, 'src-tauri/tauri.conf.json');
const conf = JSON.parse(readFileSync(tauriConf, 'utf8'));
conf.version = version;
writeFileSync(tauriConf, JSON.stringify(conf, null, 2) + '\n', 'utf8');
console.log(`tauri.conf.json => ${version}`);

// Cargo.toml (最初の version = "..." 行のみ置換)
const cargoPath = resolve(root, 'src-tauri/Cargo.toml');
const cargo = readFileSync(cargoPath, 'utf8');
const updated = cargo.replace(
  /^(version\s*=\s*)"[^"]+"/m,
  `$1"${version}"`
);
writeFileSync(cargoPath, updated, 'utf8');
console.log(`Cargo.toml    => ${version}`);
