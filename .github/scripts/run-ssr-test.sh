#!/bin/bash

FASTN_ROOT=$(pwd)
FASTN_SERVE_PID=""

terminate_fastn_serve() {
    echo "Terminating fastn serve process ..."
    if [ -n "$FASTN_SERVE_PID" ]; then
        kill -9 "$FASTN_SERVE_PID"
    fi
}

handle_error() {
    local ret=$?
    echo "Error occurred, terminating script..."
    terminate_fastn_serve
    exit $ret
}

trap 'handle_error' ERR

echo "Installing required python dependencies ..."
pip install pyquery requests

cd "$FASTN_ROOT/ssr-test"

echo "Updating fastn packages ..."
fastn update

echo "Starting fastn server in the background..."
fastn serve --offline &
FASTN_SERVE_PID=$!

# Wait for the server to start
sleep 5

echo "Running tests ..."
# Run the Python script and catch assertion errors
if python "${FASTN_ROOT}/.github/scripts/test-ssr.py" "http://127.0.0.1:8000"; then
    echo "SSR test passed successfully"
else
    echo "SSR test failed with assertion error"
    exit 1
fi

terminate_fastn_serve
