# fantasy-sport-api-rust
Core fantasy sports api in rust

# Running
Just go into any server folder and `reset && cargo run`

# Notes
Have "master" api for users, which routes between separate
- fantasy-league server
- predictions server
- results server (handles connections between teams, matches, series etc. it doesnt do 'mapping' (i.e. combining data-sources and matching up teams/matches).its up to api-user to deal with that shit and prepare queries to this with their decided team-code, match-id etc)
- leaderboard server (think it's very nice to have this separate as can run leaderboards for lots of things, doesnt just have to be fantasy-league)

separated because can run fantasy league with no predictions, can run predictions without fantasy league, can do both.

The master api would then also handle, "hey this prediction was correct. so reward fantasy user X with more money"

If want to support draft fantasy league, need websocket support out-the-box.
