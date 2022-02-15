set -xe

# We have to build the binding at least once to generate some files
cd $JIKESRVM_PATH
./bin/buildit localhost RBaseBaseNoGC -j $JAVA_HOME --answer-yes --use-third-party-heap=$BINDING_PATH --use-third-party-build-configs=$BINDING_PATH/jikesrvm/build/configs --use-external-source=$BINDING_PATH/jikesrvm/rvm/src --m32

export RUSTFLAGS="-D warnings"

cd $BINDING_PATH/mmtk
cargo clippy --target i686-unknown-linux-gnu --features nogc
cargo clippy --target i686-unknown-linux-gnu --features semispace
cargo clippy --target i686-unknown-linux-gnu --features marksweep

cargo clippy --target i686-unknown-linux-gnu --features nogc --release
cargo clippy --target i686-unknown-linux-gnu --features semispace --release
cargo clippy --target i686-unknown-linux-gnu --features marksweep --release

cargo fmt -- --check