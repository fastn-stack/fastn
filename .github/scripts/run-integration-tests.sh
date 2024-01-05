#!/bin/bash
export FASTN_ROOT=`pwd`

# Enable xtrace and pipefail
set -o xtrace
set -eou pipefail

echo "Installing python server dependencies"
pip install Flask psycopg2

echo "Waiting for postgres to be ready"
timeout=30
until pg_isready -h localhost -p 5432 -U testuser -d testdb || ((timeout-- <= 0)); do
  sleep 1
done

echo "Populating test data"
python "${FASTN_ROOT}/.github/scripts/populate-table.py"

echo "Starting test python server"
python "${FASTN_ROOT}/.github/scripts/test-server.py" &
# Waiting for the server to start
sleep 10

echo "Running integration tests"
cd "${FASTN_ROOT}/integration-tests" || exit 1
fastn test --headless
