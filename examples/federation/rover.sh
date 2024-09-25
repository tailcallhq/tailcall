#!/usr/bin/env bash

set -eumo pipefail

function cleanup {
  for pid in "${USER_ROVER_PID:-}"; do
    # try kill all registered pids
    [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null && kill "$pid" || echo "Could not kill $pid"
  done
}
trap cleanup EXIT

rover dev --url http://localhost:8001/graphql --name post &
sleep 1
rover dev --url http://localhost:8002/graphql --name user &
USER_ROVER_PID=$!
sleep 1
fg %1
