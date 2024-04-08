BINDING_PATH=$(realpath $(dirname "$0"))/../..
JIKESRVM_PATH=$BINDING_PATH/repos/jikesrvm
DACAPO_PATH=$JIKESRVM_PATH/dacapo

RUSTUP_TOOLCHAIN=`cat $BINDING_PATH/mmtk/rust-toolchain`
# We have to export this. JikesRVM build script will access this.
export RUSTUP_TOOLCHAIN=$RUSTUP_TOOLCHAIN

# We need to use Java8 or a previous version (with Java 1.5/1.6 support)
# We have to set both JAVA_HOME and PATH.
# This JDK path here work for Github hosted runner ubuntu-22.04. If
# we run on other images/machines, we need to update this path.
export JAVA_HOME=/usr/lib/jvm/temurin-8-jdk-amd64
export PATH=/usr/lib/jvm/temurin-8-jdk-amd64/bin:$PATH

export MMTK_THREADS=1
