#!/bin/bash
diesel setup --database-url postgres://fantasy:fantasy@localhost/leaderboard && 
diesel migration run --database-url postgres://fantasy:fantasy@localhost/leaderboard &&
./leaderboard_server
