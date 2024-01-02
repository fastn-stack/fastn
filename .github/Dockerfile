FROM ekidd/rust-musl-builder

# We need to add the source code to the image because `rust-musl-builder`
# assumes a UID of 1000
ADD --chown=rust:rust . ./

RUN sudo chown rust -R /opt/rust

RUN rustup target add x86_64-unknown-linux-musl

CMD RUSTFLAGS="-Clink-arg=-Wl,--allow-multiple-definition" cargo build --release --features=auth
