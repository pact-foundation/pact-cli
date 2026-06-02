#!/usr/bin/env bash

# BEFORE SUITE start mock service
# invoked by the pact framework
BIN=${BIN:-pact}
$BIN mock start \
  --port 1234 \
  --loglevel ${LOG_LEVEL:-debug} \
  --output ./tmp \
  --base-port 8081 &
pid=$!

# # BEFORE SUITE wait for mock service to start up
# # invoked by the pact framework
_wait=0
while [ "200" -ne "$(curl -s -o /dev/null -w "%{http_code}" localhost:1234)" ]; do
  sleep 0.5; _wait=$((_wait+1))
  [ $_wait -lt 60 ] || { echo "ERROR: timed out waiting for mock service on :1234"; exit 1; }
done

# # uncomment this line to see the curl commands interleaved with the responses
# # set -x

# # BEFORE EACH TEST create new mock server
# # invoked by the pact framework

# # Use cli to create mock server from pact file
mock_output=$($BIN mock create --file examples/foo-bar.json --port 1234)
# mock_output=$(pact-cli mock create --file examples/foo-bar.json --port 1234 --specification v3)
mock_id=$(echo "$mock_output" | awk '/Mock server/ {print $3}')
mock_port=$(echo "$mock_output" | awk '/Mock server/ {print $7}')
# echo "Mock server ID: $mock_id"
# echo "Mock server port: $mock_port"


# # BEFORE A TEST set up interaction(s) just for that test
# # The contents of this would be written by the developer in the provided pact DSL for
# # your language eg. mockService.given(...).uponReceiving(...). ...
# # This can be called mulitple times. Alternatively PUT could be used
# # with a body of `{interactions: [...]}` which would negate the need to call DELETE.
# use rest api to create mock server from pact file
# response=$(curl -s -X POST -H "Content-Type: application/json" -d @examples/foo-bar.json localhost:1234)
# mock_id=$(echo "$response" | jq -r '.mockServer.id')
# mock_port=$(echo "$response" | jq -r '.mockServer.port')
# echo "Mock server ID: $mock_id"
# echo "Mock server port: $mock_port"

# # BEFORE SUITE wait for mock server to start up
# # invoked by the pact framework
_wait=0
while [ "200" -ne "$(curl -s -o /dev/null -w "%{http_code}" localhost:1234/mockserver/$mock_id)" ]; do
  sleep 0.5; _wait=$((_wait+1))
  [ $_wait -lt 60 ] || { echo "ERROR: timed out waiting for mock server $mock_id on :1234"; exit 1; }
done

# # get details of the newly created mock server via api
# curl -s -H "Content-Type: application/json" localhost:1234/mockserver/$mock_id | jq .

# # get info about all mock servers via cli
# # no mechanism to list details about indiv mock server via cli
# $BIN mock list --port 1234
# # get info about all mock servers via rest api
# curl -s -H "Content-Type: application/json" localhost:1234 | jq .


# check the status of the mock server via api is ok
mock_server_status=$(curl -s -H "Content-Type: application/json" localhost:1234/mockserver/$mock_id)
# Extract fields from mock_server_status using jq and store in variables
mock_address=$(echo "$mock_server_status" | jq -r '.address')
mock_requests=$(echo "$mock_server_status" | jq -r '.metrics.requests')
mock_provider_name=$(echo "$mock_server_status" | jq -r '.provider')
mock_scheme=$(echo "$mock_server_status" | jq -r '.scheme')
mock_status=$(echo "$mock_server_status" | jq -r '.status')
echo "Mock server address: $mock_address"
echo "Mock server requests: $mock_requests"
echo "Mock server provider name: $mock_provider_name"
echo "Mock server scheme: $mock_scheme"
echo "Mock server status: $mock_status"

# # IN A TEST execute interaction(s)
# # this would be done by the consumer code under test
curl $mock_scheme://$mock_address/foo
echo ''

# # AFTER EACH TEST verify interaction(s) took place
# # This would be done explicitly by the developer or automatically by the framework,
# # depending on the language
# verify interactions took place via rest api
curl -s -X POST -H "Content-Type: application/json" localhost:1234/mockserver/$mock_id/verify | jq .

# check the status of the mock server via api is ok
mock_server_status=$(curl -s -H "Content-Type: application/json" localhost:1234/mockserver/$mock_id)
# Extract fields from mock_server_status using jq and store in variables
mock_address=$(echo "$mock_server_status" | jq -r '.address')
mock_requests=$(echo "$mock_server_status" | jq -r '.metrics.requests')
mock_provider_name=$(echo "$mock_server_status" | jq -r '.provider')
mock_scheme=$(echo "$mock_server_status" | jq -r '.scheme')
mock_status=$(echo "$mock_server_status" | jq -r '.status')
echo "Mock server address: $mock_address"
echo "Mock server requests: $mock_requests"
echo "Mock server provider name: $mock_provider_name"
echo "Mock server scheme: $mock_scheme"
echo "Mock server status: $mock_status"

# # use rest api to shutdown mock server by id
# curl -s -X DELETE -H "Content-Type: application/json" localhost:1234/mockserver/$mock_id
# use cli to shutdown mock server by id or port
$BIN mock shutdown --mock-server-id $mock_id --port 1234
# $BIN mock shutdown --mock-server-port $mock_port --port 1234

# # AFTER SUITE stop mock service
# # this would be invoked by the test framework
kill -9 $pid

while [ kill -0 $pid 2> /dev/null ]; do sleep 0.5; done

# echo ''
# echo 'FYI the mock service logs are:'
# cat ./tmp/bar_mock_service.log