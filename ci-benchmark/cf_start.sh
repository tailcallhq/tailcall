#!/bin/bash

url="http://localhost:19194/\?config\=https://raw.githubusercontent.com/tailcallhq/tailcall/main/ci-benchmark/benchmark.graphql"
while true; do
    response_code=$(curl -s -o /dev/null -w "%{http_code}" "$url")

    if [ "$response_code" -eq 200 ]; then
        break
    else
        echo "Waiting for Cloudflare Worker to be ready..."
        sleep 10
    fi
done
