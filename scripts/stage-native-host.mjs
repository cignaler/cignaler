/**
 * Copies the built cignaler-native-host binary to src-tauri/binaries/<name>-<host-triple>
 * so that Tauri's externalBin bundler can include it in the app bundle.
 *
 * Usage:
 *   node scripts/stage-native-host.mjs          (release profile, default)
 *   node scripts/stage-native-host.mjs debug    (debug profile)
 */

import { execSync } from 'child_process';
import { copyFileSync, mkdirSync, chmodSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const profile = process.argv[2] === 'debug' ? 'debug' : 'release';
const root = dirname(dirname(fileURLToPath(import.meta.url)));

const hostOutput = execSync('rustc -vV').toString();
const match = hostOutput.match(/host:\s+(.+)/);
if (!match) {
    console.error('Could not determine host target triple from rustc -vV');
    process.exit(1);
}
const triple = match[1].trim();

const isWindows = process.platform === 'win32';
const ext = isWindows ? '.exe' : '';
const binaryName = `cignaler-native-host${ext}`;

const srcPath = join(root, 'src-tauri', 'target', profile, binaryName);
const destDir = join(root, 'src-tauri', 'binaries');
const destPath = join(destDir, `cignaler-native-host-${triple}${ext}`);

if (!existsSync(srcPath)) {
    console.error(`Source binary not found: ${srcPath}`);
    console.error('Run: cargo build --features skip-tauri-build --bin cignaler-native-host');
    process.exit(1);
}

mkdirSync(destDir, { recursive: true });
copyFileSync(srcPath, destPath);

if (!isWindows) {
    chmodSync(destPath, 0o755);
}

console.log(`Staged native host: ${destPath}`);
