FROM rustlang/rust:nightly

WORKDIR /opt/project

ENV USER=root

RUN cargo new blockchain-rs \
    && cd blockchain-rs \
    && cargo build --release

COPY . .

RUN cargo build --release

ENTRYPOINT ["/opt/project/target/release/blockchain-rs"]
