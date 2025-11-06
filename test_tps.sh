cd integration_tests
export NSSA_WALLET_HOME_DIR=$(pwd)/configs/debug/wallet/
RUST_LOG=info cargo run --release $(pwd)/configs/debug tps_test
