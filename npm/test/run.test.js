'use strict';

const { test, before, after } = require('node:test');
const assert = require('node:assert/strict');
const { spawn, spawnSync } = require('node:child_process');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');

const { platformPackage } = require('../pact-cli/lib/run');

const PACKAGE_DIR = path.join(__dirname, '..', 'pact-cli');

let fixtureRoot;

// Builds an install tree in which `node` stands in for the `pact` binary, so
// `pact <script> ...args` runs `<script>` from the fixture's cwd. The tree
// sits under a path with a space, which every test spawns through.
function makeFixture({ withPlatformPackage }) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'pact npm '));
  const modules = path.join(root, 'node_modules');
  fs.cpSync(PACKAGE_DIR, path.join(modules, '@pact-foundation', 'pact-cli'), { recursive: true });

  if (withPlatformPackage) {
    const pkg = platformPackage(process.platform, process.arch);
    const pkgDir = path.join(modules, ...pkg.split('/'));
    fs.mkdirSync(path.join(pkgDir, 'standalone'), { recursive: true });
    fs.writeFileSync(path.join(pkgDir, 'package.json'), JSON.stringify({ name: pkg }));
    if (process.platform === 'win32') {
      fs.copyFileSync(process.execPath, path.join(pkgDir, 'standalone', 'pact.exe'));
    } else {
      fs.symlinkSync(process.execPath, path.join(pkgDir, 'standalone', 'pact'));
    }
  }

  fs.writeFileSync(path.join(root, 'echo-exit3.js'), 'console.log(JSON.stringify(process.argv.slice(2)));\nprocess.exit(3);\n');
  fs.writeFileSync(path.join(root, 'sigterm.js'), "process.kill(process.pid, 'SIGTERM');\n");
  // Node handles SIGUSR1 itself, so the script execs a shell, which dies from it.
  fs.writeFileSync(path.join(root, 'sigusr1.js'), "process.execve('/bin/sh', ['sh', '-c', 'kill -USR1 $$']);\n");
  // Writes its pid to the file named by its first argument, then runs until signalled.
  fs.writeFileSync(
    path.join(root, 'long.js'),
    "require('fs').writeFileSync(process.argv[2], String(process.pid));\nsetInterval(() => {}, 1000);\n",
  );
  return root;
}

function pactBin(root) {
  return path.join(root, 'node_modules', '@pact-foundation', 'pact-cli', 'bin', 'pact.js');
}

function runPact(root, args) {
  return spawnSync(process.execPath, [pactBin(root), ...args], { cwd: root, encoding: 'utf8' });
}

function isAlive(pid) {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    return error.code !== 'ESRCH';
  }
}

async function waitFor(predicate, timeoutMs = 5000) {
  const deadline = Date.now() + timeoutMs;
  while (!predicate()) {
    if (Date.now() > deadline) throw new Error('timed out');
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
}

before(() => {
  fixtureRoot = makeFixture({ withPlatformPackage: true });
});

after(() => {
  fs.rmSync(fixtureRoot, { recursive: true, force: true });
});

test('forwards args and exit status', () => {
  const result = runPact(fixtureRoot, ['echo-exit3.js', 'a', 'b c']);
  assert.equal(result.status, 3, result.stderr);
  assert.deepEqual(JSON.parse(result.stdout), ['a', 'b c']);
});

test('re-raises child signal', { skip: process.platform === 'win32' }, () => {
  const result = runPact(fixtureRoot, ['sigterm.js']);
  assert.equal(result.signal, 'SIGTERM');
  assert.equal(result.status, null);
});

// The wrapper's own Node answers SIGUSR1 by starting the inspector, so
// re-raising it cannot end the wrapper.
test('exits non-zero when the re-raised signal does not end the wrapper', {
  skip: process.platform === 'win32' || typeof process.execve !== 'function',
}, () => {
  const result = runPact(fixtureRoot, ['sigusr1.js']);
  assert.equal(result.status, 128 + os.constants.signals.SIGUSR1, result.stderr);
});

test('forwards a signal sent to the wrapper', { skip: process.platform === 'win32' }, async () => {
  const pidFile = path.join(fixtureRoot, 'child.pid');
  const wrapper = spawn(process.execPath, [pactBin(fixtureRoot), 'long.js', pidFile], {
    cwd: fixtureRoot,
    stdio: 'ignore',
  });
  const exited = new Promise((resolve) => wrapper.on('exit', (code, signal) => resolve({ code, signal })));

  await waitFor(() => fs.existsSync(pidFile) && fs.readFileSync(pidFile, 'utf8') !== '');
  const childPid = Number(fs.readFileSync(pidFile, 'utf8'));
  wrapper.kill('SIGTERM');

  assert.deepEqual(await exited, { code: null, signal: 'SIGTERM' });
  await waitFor(() => !isAlive(childPid));
});

test('missing platform package', () => {
  const root = makeFixture({ withPlatformPackage: false });
  try {
    const result = runPact(root, ['--help']);
    assert.equal(result.status, 1);
    assert.ok(result.stderr.includes(platformPackage(process.platform, process.arch)), result.stderr);
    assert.ok(!result.stderr.includes('    at '), result.stderr);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
