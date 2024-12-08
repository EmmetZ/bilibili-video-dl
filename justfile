default:
    @just -l

build: check
    cargo build --release

test target="":
    cargo test --package bili-dl --bin bili-dl -- {{target}} --show-output 

run:
    cargo run -- https://www.bilibili.com/video/BV1zhYJeFELy

run-c:
    cargo run -- https://www.bilibili.com/video/BV1zhYJeFELy -c ./cookie.txt

run-b:
    cargo run -- https://www.bilibili.com/bangumi/media/md963 -c ./cookie.txt 

check:
    cargo fmt && cargo check

help:
    cargo run -- --help
