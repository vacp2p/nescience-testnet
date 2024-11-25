set -e

curl -L https://risczero.com/install | bash 
/home/runner/.risc0/bin/rzup install 
cargo install taplo-cli --locked

cargo fmt -- --check

cd accounts
cargo clippy --release -- -D warnings
cd ..

cd consensus
cargo clippy --release -- -D warnings
cd ..

cd mempool
cargo clippy --release -- -D warnings
cd ..

cd networking
cargo clippy --release -- -D warnings
cd ..

cd rpc_primitives
cargo clippy --release -- -D warnings
cd ..

cd sequencer_core
cargo clippy --release -- -D warnings
cd ..

cd sequencer_rpc
cargo clippy --release -- -D warnings
cd ..

cd sequencer_runner
cargo clippy --release -- -D warnings
cd ..

cd storage
cargo clippy --release -- -D warnings
cd ..

cd utxo
cargo clippy --release -- -D warnings
cd ..

taplo fmt --check