find . | grep rs$ | entr -d sh -c "time RUST_BACKTRACE=1 timeout 20 cargo run"
