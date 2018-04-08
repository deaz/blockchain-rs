# blockchain-rs

Rust implementation of [naivechain](https://github.com/lhartikk/naivechain).

## Run

`docker-compose up`

## Add block

`curl --data "data=testdata" localhost:8000/mine-block`

## Show blocks

`curl localhost:8000/blocks`

## Add peer

`curl --data "peer=ws://node2:3012" localhost:8000/add-peer`

## Show peers

`curl localhost:8000/peers`
