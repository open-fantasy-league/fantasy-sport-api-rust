# https://blog.sedrik.se/posts/my-docker-setup-for-rust/
FROM ekidd/rust-musl-builder:latest as builder
USER rust
# Face and palm are in alignment https://github.com/moby/moby/issues/36408
RUN mkdir -p /home/rust/fantasy/src
WORKDIR /home/rust/fantasy

# Avoid having to install/build all dependencies by copying
# the Cargo files and making a dummy src/main.rs
COPY --chown=rust:rust Cargo.* ./

# These are now installed through cargo.toml github
# Just to get diesel_utils and warp_ws_server onto container. In future these can be published crates
# ADD https://api.github.com/repos/open-fantasy-league/fantasy-sport-api-rust/git/refs/heads/master version.json
# RUN git clone -b master https://github.com/open-fantasy-league/fantasy-sport-api-rust.git /home/rust/fantasy-sport-api-rust
# RUN mv /home/rust/fantasy-sport-api-rust/diesel_utils /home/rust/diesel_utils
# RUN mv /home/rust/fantasy-sport-api-rust/ws_server /home/rust/ws_server

RUN echo "fn main() {}" > /home/rust/fantasy/src/main.rs
RUN cargo build

# We need to touch our real main.rs file or else docker will use
# the cached one.
COPY --chown=rust:rust . .
RUN touch /home/rust/fantasy/src/main.rs

# RUN cargo test
# Current diesel_cli compile on rust-musl is borken
# So Im copying my local already built diesel binary
#RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo build

# Size optimization
#RUN strip target/x86_64-unknown-linux-musl/release/fantasy_server

# Start building the final image

# Scratch is awkward as doesnt have /bash or /sh, so its hard to chain the 3 commands needed on start
# (Have to wait for pg to be started before running migrations, cant do in the building)
#FROM scratch
#WORKDIR /home/rust/fantasy
#COPY --from=builder /home/rust/fantasy/target/x86_64-unknown-linux-musl/debug/fantasy_server .
#COPY --from=builder /home/rust/fantasy/diesel-cli-bin /home/rust/fantasy/diesel
RUN cp /home/rust/fantasy/target/x86_64-unknown-linux-musl/debug/fantasy_server .
RUN mv diesel-cli-bin diesel
#ENTRYPOINT ["./run.sh"]
