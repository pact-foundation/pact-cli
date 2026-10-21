#!/usr/bin/env node
// Repackages the cargo-dist release archives as `@pact-foundation/pact-cli`
// npm packages.
//
// Writes one directory per platform package, holding the `pact` binary at
// `standalone/pact` (`standalone/pact.exe` on Windows), and one directory for
// the main package, which depends on all of them as optional dependencies. The
// binary is copied byte for byte from the release archive.
//
// Prints each package directory, platform packages first and the main package
// last, which is the order they must be published in.
//
// Node has no xz or zip support, so extraction uses `tar` and `unzip`.
//
// Usage: build-npm.js --version 18.10.9 --archives DIR --wrapper npm/pact-cli --license LICENSE --out DIR

'use strict';

const { execFileSync } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');
const { parseArgs } = require('node:util');

const SCOPE = '@pact-foundation';
const MAIN_NAME = 'pact-cli';
const REPOSITORY = { type: 'git', url: 'git+https://github.com/pact-foundation/pact-cli.git' };

// The Windows ARM64 package carries the x86_64 binary, which runs under Windows'
// x64 emulation until `aarch64-pc-windows-msvc` is re-enabled in Cargo.toml.
const TARGETS = {
  'aarch64-apple-darwin': [['darwin', 'darwin', 'arm64']],
  'x86_64-apple-darwin': [['darwin', 'darwin', 'x64']],
  'aarch64-unknown-linux-musl': [['linux', 'linux', 'arm64']],
  'x86_64-unknown-linux-musl': [['linux', 'linux', 'x64']],
  'x86_64-pc-windows-msvc': [
    ['windows', 'win32', 'x64'],
    ['windows', 'win32', 'arm64'],
  ],
};

function archivePath(archives, target) {
  const ext = target.includes('windows') ? 'zip' : 'tar.xz';
  return path.join(archives, `pact-${target}.${ext}`);
}

function readBinary(archives, target) {
  const archive = archivePath(archives, target);
  const options = { maxBuffer: 512 * 1024 * 1024 };
  if (target.includes('windows')) {
    return { name: 'pact.exe', data: execFileSync('unzip', ['-p', archive, 'pact.exe'], options) };
  }
  return { name: 'pact', data: execFileSync('tar', ['-xJOf', archive, `pact-${target}/pact`], options) };
}

function writeJson(file, data) {
  fs.writeFileSync(file, `${JSON.stringify(data, null, 2)}\n`);
}

function writePlatformPackage(out, version, [osName, npmOs, cpu], binary, license) {
  const packageName = `${SCOPE}/${MAIN_NAME}-${osName}-${cpu}`;
  const packageDir = path.join(out, `${MAIN_NAME}-${osName}-${cpu}`);
  const standalone = path.join(packageDir, 'standalone');
  fs.mkdirSync(standalone, { recursive: true });

  const binaryPath = path.join(standalone, binary.name);
  fs.writeFileSync(binaryPath, binary.data);
  fs.chmodSync(binaryPath, 0o755);
  fs.writeFileSync(path.join(packageDir, 'LICENSE'), license);
  writeJson(path.join(packageDir, 'package.json'), {
    name: packageName,
    version,
    description: `pact binary for ${SCOPE}/${MAIN_NAME} (${osName}-${cpu})`,
    license: 'MIT',
    repository: REPOSITORY,
    os: [npmOs],
    cpu: [cpu],
    files: ['standalone'],
  });
  return { packageName, packageDir };
}

function writeMainPackage(out, version, wrapper, license, platformNames) {
  const packageDir = path.join(out, MAIN_NAME);
  fs.cpSync(wrapper, packageDir, { recursive: true });
  fs.writeFileSync(path.join(packageDir, 'LICENSE'), license);

  const manifestPath = path.join(packageDir, 'package.json');
  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  manifest.version = version;
  manifest.optionalDependencies = Object.fromEntries(platformNames.map((name) => [name, version]));
  writeJson(manifestPath, manifest);
  return packageDir;
}

function main() {
  const { values: args } = parseArgs({
    options: {
      version: { type: 'string' },
      archives: { type: 'string' },
      wrapper: { type: 'string' },
      license: { type: 'string' },
      out: { type: 'string' },
    },
  });
  for (const name of ['version', 'archives', 'wrapper', 'license', 'out']) {
    if (!args[name]) {
      console.error(`build-npm.js: --${name} is required`);
      process.exit(2);
    }
  }

  const missing = Object.keys(TARGETS)
    .map((target) => archivePath(args.archives, target))
    .filter((file) => !fs.existsSync(file));
  if (missing.length > 0) {
    for (const file of missing) {
      console.error(
        `missing cargo-dist archive ${path.basename(file)} in ${args.archives}; tags before v0.10.x lack these`,
      );
    }
    process.exit(1);
  }

  const license = fs.readFileSync(args.license);
  fs.mkdirSync(args.out, { recursive: true });

  const platformNames = [];
  for (const [target, packages] of Object.entries(TARGETS)) {
    const binary = readBinary(args.archives, target);
    for (const names of packages) {
      const { packageName, packageDir } = writePlatformPackage(args.out, args.version, names, binary, license);
      platformNames.push(packageName);
      console.log(packageDir);
    }
  }

  console.log(writeMainPackage(args.out, args.version, args.wrapper, license, platformNames));
}

main();
