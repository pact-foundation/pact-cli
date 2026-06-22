#!/bin/sh -e

## Deprecation shim — delegates to the cargo-dist-generated installer.
##
## DEPRECATED: will be removed after 2027-01-01.
## Use the new installer instead:
##
##   curl --proto '=https' --tlsv1.2 -LsSf \
##     https://github.com/pact-foundation/pact-cli/releases/latest/download/pact-installer.sh | sh
##
## To install a specific version:
##   curl --proto '=https' --tlsv1.2 -LsSf \
##     https://github.com/pact-foundation/pact-cli/releases/download/<VERSION>/pact-installer.sh | sh

cat >&2 <<EOF
DEPRECATION NOTICE

This install script is deprecated and will stop working after 2027-01-01.
Use the new installer instead:

  curl --proto '=https' --tlsv1.2 -LsSf \
    https://github.com/pact-foundation/pact-cli/releases/latest/download/pact-installer.sh | sh
EOF

if [ -z "$PACT_CLI_VERSION" ] || [ "$PACT_CLI_VERSION" = "vlatest" ]; then
	installer_url="https://github.com/pact-foundation/pact-cli/releases/latest/download/pact-installer.sh"
else
	installer_url="https://github.com/pact-foundation/pact-cli/releases/download/${PACT_CLI_VERSION}/pact-installer.sh"
fi

tmpfile=$(mktemp /tmp/pact-installer.XXXXXX) || exit 1
trap 'rm -f "$tmpfile"' EXIT

if command -v curl >/dev/null 2>&1; then
	if ! curl --proto '=https' --tlsv1.2 -LsSf --fail -o "$tmpfile" "$installer_url"; then
		echo "Failed to download installer from: $installer_url" >&2
		echo "Versions older than v0.8.0 do not have a cargo-dist installer." >&2
		echo "Download manually from: https://github.com/pact-foundation/pact-cli/releases/tag/${PACT_CLI_VERSION:-latest}" >&2
		exit 1
	fi
elif command -v wget >/dev/null 2>&1; then
	if ! wget -q -O "$tmpfile" "$installer_url"; then
		echo "Failed to download installer from: $installer_url" >&2
		echo "Versions older than v0.8.0 do not have a cargo-dist installer." >&2
		echo "Download manually from: https://github.com/pact-foundation/pact-cli/releases/tag/${PACT_CLI_VERSION:-latest}" >&2
		exit 1
	fi
else
	echo "Error: curl or wget is required to install pact." >&2
	exit 1
fi

sh "$tmpfile"
