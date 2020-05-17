#!/bin/bash
set -e

echo "Running extensions"
# psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" <<-EOSQL
# CREATE EXTENSION IF NOT EXISTS pgcrypto;
# CREATE EXTENSION IF NOT EXISTS btree_gist;
# CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;
# EOSQL