FROM ekidd/rust-musl-builder

# We need to add the source code to the image because `rust-musl-builder`
# assumes a UID of 1000
ADD --chown=rust:rust . ./

CMD RUSTFLAGS="-Clink-arg=-Wl,--allow-multiple-definition" cargo build --release --features=auth
