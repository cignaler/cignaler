/**
 * Builds cignaler-native-host and stages it at
 * src-tauri/binaries/cignaler-native-host-<target-triple>[.exe], which is where
 * Tauri's externalBin bundler looks for it.
 *
 * The target triple comes from TAURI_ENV_TARGET_TRIPLE (Tauri sets this on
 * build hooks), else `--target <triple>`, else the rustc host triple.
 *
 * universal-apple-darwin is special: Tauri lipos only the *main* binary, so a
 * sidecar has to arrive already universal. We build both slices and lipo here.
 *
 * Usage:
 *   node scripts/stage-native-host.mjs                              (release)
 *   node scripts/stage-native-host.mjs debug                        (debug)
 *   node scripts/stage-native-host.mjs --target universal-apple-darwin
 */

import { execFileSync, execSync } from 'child_process';
import { mkdirSync, chmodSync, copyFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const UNIVERSAL = 'universal-apple-darwin';
const UNIVERSAL_SLICES = ['aarch64-apple-darwin', 'x86_64-apple-darwin'];

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const manifest = join(root, 'src-tauri', 'Cargo.toml');

const args = process.argv.slice(2);
const profile = args.includes('debug') ? 'debug' : 'release';

function hostTriple() {
    const match = execSync('rustc -vV').toString().match(/host:\s+(.+)/);
    if (!match) {
        console.error('Could not determine host target triple from `rustc -vV`');
        process.exit(1);
    }
    return match[1].trim();
}

const targetFlag = args.indexOf('--target');
const target =
    process.env.TAURI_ENV_TARGET_TRIPLE ||
    (targetFlag !== -1 ? args[targetFlag + 1] : undefined) ||
    hostTriple();

// lipo only exists on macOS; a universal build anywhere else is a mistake.
if (target === UNIVERSAL && process.platform !== 'darwin') {
    console.error(`${UNIVERSAL} can only be built on macOS`);
    process.exit(1);
}

const ext = process.platform === 'win32' ? '.exe' : '';
const binaryName = `cignaler-native-host${ext}`;
const destDir = join(root, 'src-tauri', 'binaries');
const destPath = join(destDir, `cignaler-native-host-${target}${ext}`);

const host = hostTriple();

/**
 * Builds one slice and returns the path to the produced binary. Building for
 * the host triple omits --target so the artifacts land in the same target dir
 * Tauri itself uses — passing it would force a full recompile of the
 * dependency tree into a second directory.
 */
function build(triple) {
    const cross = triple !== host;
    const cargoArgs = [
        'build',
        '--manifest-path', manifest,
        '--features', 'skip-tauri-build',
        '--bin', 'cignaler-native-host',
    ];
    if (profile === 'release') cargoArgs.push('--release');
    if (cross) cargoArgs.push('--target', triple);

    console.log(`cargo ${cargoArgs.join(' ')}`);
    execFileSync('cargo', cargoArgs, { stdio: 'inherit', cwd: root });

    const outDir = cross
        ? join(root, 'src-tauri', 'target', triple, profile)
        : join(root, 'src-tauri', 'target', profile);
    return join(outDir, binaryName);
}

mkdirSync(destDir, { recursive: true });

const staged = [];

if (target === UNIVERSAL) {
    // A universal build needs all three sidecars on disk. Tauri compiles each
    // arch separately before lipo-ing the main binary, and tauri-build asserts
    // a sidecar exists for *that slice's* triple; the bundler then picks the
    // universal one to copy into the .app.
    const slices = UNIVERSAL_SLICES.map((triple) => {
        const built = build(triple);
        const slicePath = join(destDir, `cignaler-native-host-${triple}`);
        copyFileSync(built, slicePath);
        staged.push(slicePath);
        return built;
    });
    execFileSync('lipo', ['-create', '-output', destPath, ...slices], { stdio: 'inherit' });
} else {
    copyFileSync(build(target), destPath);
}

staged.push(destPath);

if (process.platform !== 'win32') {
    for (const path of staged) chmodSync(path, 0o755);
}

console.log(`Staged native host (${target}):`);
for (const path of staged) console.log(`  ${path}`);
