FROM rustlang/rust:nightly

WORKDIR /opt/project
ENV USER=root
RUN cargo new blockchain-rs

# TODO: use /opt/project as root for crate
WORKDIR blockchain-rs
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release
# for development build cache in ./target/debug
RUN cargo build

COPY . .

RUN cargo build --release

ENTRYPOINT ["/opt/project/target/release/blockchain-rs"]
