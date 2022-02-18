BINDING_PATH=$(realpath $(dirname "$0"))/../..
JIKESRVM_PATH=$BINDING_PATH/repos/jikesrvm
DACAPO_PATH=$JIKESRVM_PATH/dacapo

RUSTUP_TOOLCHAIN=`cat $BINDING_PATH/mmtk/rust-toolchain`
# We have to export this. JikesRVM build script will access this.
export RUSTUP_TOOLCHAIN=$RUSTUP_TOOLCHAIN
