# @pact-foundation/pact-cli

The [Pact](https://docs.pact.io) command-line tool, packaged for npm. It ships the
native `pact` binary from [pact-foundation/pact-cli](https://github.com/pact-foundation/pact-cli)
for macOS, Linux and Windows on x64 and arm64.

```sh
npm install --save-dev @pact-foundation/pact-cli
npx pact --help
```

## Commands

Every command runs as `pact <command>`: `pact broker`, `pact mock`,
`pact stub`, `pact plugin`, `pact verifier` and `pact pactflow`. Run
`npx pact --help` for the full list.

The hyphenated commands `pact-broker`, `pact-mock-server`, `pact-stub-server`,
`pact-plugin`, `pact-verifier` and `pactflow` still work, but print a
deprecation notice to stderr. They will be removed in a future major release.

The binary comes from a platform package such as
`@pact-foundation/pact-cli-linux-x64`, installed as an optional dependency.
Installs with `--no-optional` or `--omit=optional` leave the commands unable to
find it.

Versions 17.x and earlier shipped a JavaScript API and the Ruby standalone
tools. See the
[migration guide](https://github.com/pact-foundation/pact-js-cli/blob/main/MIGRATION.md).

Usage and documentation: <https://github.com/pact-foundation/pact-cli#readme>
