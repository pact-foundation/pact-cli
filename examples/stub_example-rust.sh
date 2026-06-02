#!/usr/bin/env bash

# BEFORE SUITE start mock service
# invoked by the pact framework
BIN=${BIN:-pact}
$BIN stub --file examples/foo-bar.json --file examples/stub-health-check.json --port 1234 \
  --loglevel ${LOG_LEVEL:-debug} &
pid=$!

# BEFORE SUITE wait for mock service to start up
# invoked by the pact framework
_wait=0
while [ "200" -ne "$(curl -H "X-Pact-Stub-Server: true" -s -o /dev/null -w "%{http_code}" localhost:1234/healthcheck)" ]; do
  sleep 0.5; _wait=$((_wait+1))
  [ $_wait -lt 60 ] || { echo "ERROR: timed out waiting for stub server on :1234"; exit 1; }
done

# IN A TEST execute interaction(s)
# this would be done by the consumer code under test
curl localhost:1234/foo
echo ''


# AFTER SUITE stop mock service
# this would be invoked by the test framework
kill -9 $pid

while kill -0 $pid 2>/dev/null; do sleep 0.5; done