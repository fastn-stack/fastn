#!/bin/bash
export FASTN_ROOT=`pwd`
export FASTN_DB_URL=postgres://testuser:testpassword@localhost:5432/testdb

# Enable xtrace and pipefail
set -o xtrace
set -eou pipefail

echo "Installing python server dependencies"
python -m pip install --upgrade pip
pip install Flask psycopg2

echo "Waiting for postgres to be ready"
timeout=30
until pg_isready -h localhost -p 5432 -U testuser -d testdb || ((timeout-- <= 0)); do
  sleep 1
done

echo "Populating test data"
python ${FASTN_ROOT}/scripts/populate-table.py

echo "Starting test python server"
python ${FASTN_ROOT}/scripts/test-server.py &
# Waiting for the server to start
sleep 10

echo "Running integration tests"
cd ${FASTN_ROOT}/integration-tests
fastn test --headless
