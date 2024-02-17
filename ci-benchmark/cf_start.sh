#!/bin/bash

url="http://localhost:19194/"
counter=0

while true; do
    response_code=$(curl -s -o /dev/null -w "%{http_code}" "$url")

    if [ "$response_code" -eq 200 ]; then
        break
    else
        echo "Waiting for Cloudflare Worker to be ready..."
        sleep 10
        ((counter++))

        if [ "$counter" -gt 60 ]; then
            echo "Unable to start worker, exiting to prevent infinite loop"
            exit 1
        fi
    fi
done
