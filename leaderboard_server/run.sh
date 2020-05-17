#!/bin/bash
set -e
ls -la
#psql -h 'db' -c  'CREATE EXTENSION IF NOT EXISTS pgcrypto; CREATE EXTENSION IF NOT EXISTS btree_gist; CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE'
./diesel-cli-bin setup --database-url postgres://fantasy:fantasy@db/leaderboard
./diesel-cli-bin migration run --database-url postgres://fantasy:fantasy@db/leaderboard
#/bin/bash -c './diesel-cli-bin setup --database-url postgres://fantasy:fantasy@db/leaderboard' 
#/bin/bash -c './diesel-cli-bin migration run --database-url postgres://fantasy:fantasy@db/leaderboard'
./leaderboard_server
