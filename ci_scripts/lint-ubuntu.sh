set -e

curl -L https://risczero.com/install | bash 
/home/runner/.risc0/bin/rzup install 
source env.sh
cargo install taplo-cli --locked

cargo fmt -- --check
cd accounts
    taplo fmt --check
cd ..
cd common
    taplo fmt --check
cd ..
cd consensus
    taplo fmt --check
cd ..
cd mempool
    taplo fmt --check
cd ..
cd networking
    taplo fmt --check
cd ..
cd node_core
    taplo fmt --check
cd ..
cd node_rpc
    taplo fmt --check
cd ..
cd node_runner
    taplo fmt --check
cd ..
cd sc_core
    taplo fmt --check
cd ..
cd sequencer_core
    taplo fmt --check
cd ..
cd storage
    taplo fmt --check
cd ..
cd utxo
    taplo fmt --check
cd ..
cd vm
    taplo fmt --check
cd ..
cd zkvm
    taplo fmt --check
cd ..