project_root=$(dirname "$0")/../..
set -xe

export RUSTFLAGS="-D warnings"

cd $project_root/mmtk
cargo clippy --target i686-unknown-linux-gnu --features nogc
cargo clippy --target i686-unknown-linux-gnu --features semispace
cargo clippy --target i686-unknown-linux-gnu --features marksweep

cargo clippy --target i686-unknown-linux-gnu --features nogc --release
cargo clippy --target i686-unknown-linux-gnu --features semispace --release
cargo clippy --target i686-unknown-linux-gnu --features marksweep --release