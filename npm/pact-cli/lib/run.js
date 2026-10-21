'use strict';

const { spawn } = require('child_process');
const os = require('os');
const path = require('path');

// Signals a process manager or test harness sends to the wrapper's own pid.
const FORWARDED_SIGNALS = ['SIGINT', 'SIGTERM', 'SIGHUP', ...(process.platform === 'win32' ? ['SIGBREAK'] : [])];

class MissingPlatformPackage extends Error {
  constructor(packageName) {
    super(`cannot resolve ${packageName}`);
    this.name = 'MissingPlatformPackage';
    this.packageName = packageName;
  }
}

// The Windows packages were first published as `-windows-`, so the name keeps
// it even though Node reports the platform as `win32`.
function platformPackage(platform, arch) {
  const os = platform === 'win32' ? 'windows' : platform;
  return `@pact-foundation/pact-cli-${os}-${arch}`;
}

function binaryPath(platform = process.platform, arch = process.arch) {
  const pkg = platformPackage(platform, arch);
  let manifest;
  try {
    manifest = require.resolve(`${pkg}/package.json`);
  } catch (error) {
    throw new MissingPlatformPackage(pkg);
  }
  const exe = platform === 'win32' ? 'pact.exe' : 'pact';
  return path.join(path.dirname(manifest), 'standalone', exe);
}

function run(prefix) {
  let binary;
  try {
    binary = binaryPath();
  } catch (error) {
    if (!(error instanceof MissingPlatformPackage)) throw error;
    console.error(
      `pact: could not find ${error.packageName}; reinstall @pact-foundation/pact-cli ` +
        `with optional dependencies enabled (${process.platform}-${process.arch})`,
    );
    process.exit(1);
  }

  const child = spawn(binary, [...prefix, ...process.argv.slice(2)], { stdio: 'inherit' });
  const forward = (signal) => child.kill(signal);
  for (const signal of FORWARDED_SIGNALS) process.on(signal, forward);

  child.on('error', (error) => {
    console.error(`pact: ${error.message}`);
    process.exit(1);
  });
  child.on('exit', (code, signal) => {
    if (!signal) process.exit(code);
    for (const forwarded of FORWARDED_SIGNALS) process.removeListener(forwarded, forward);
    process.kill(process.pid, signal);
    // Node ignores some signals, such as SIGPIPE, so the re-raise can return.
    process.exit(128 + (os.constants.signals[signal] || 0));
  });
}

// Runs a hyphenated command such as `pact-broker` after pointing at its
// `pact <subcommand>` replacement.
function deprecated(name, prefix) {
  console.error(`warning: ${name} is deprecated; use 'pact ${prefix.join(' ')}' instead`);
  run(prefix);
}

module.exports = { MissingPlatformPackage, platformPackage, binaryPath, run, deprecated };
