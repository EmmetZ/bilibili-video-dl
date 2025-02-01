default:
    @just -l

build: check
    cargo build --release

test target="":
    cargo test --package bili-dl --bin bili-dl -- {{target}} --show-output 

run:
    cargo run -- https://www.bilibili.com/video/BV1zhYJeFELy

run-c:
    cargo run -- https://www.bilibili.com/video/BV1zhYJeFELy -f ./cookies.txt

run-b:
    cargo run -- https://www.bilibili.com/bangumi/media/md963 -f ./cookies.txt 

check:
    cargo fmt -v && cargo check

help:
    cargo run -- --help

tt:
    @time ./target/release/bili-dl -V
