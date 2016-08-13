find . | grep rs$ | entr -d sh -c "time RUST_BACKTRACE=1 cargo run config-dev-1.yaml"
