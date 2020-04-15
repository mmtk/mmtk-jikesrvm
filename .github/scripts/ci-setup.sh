set -xe

export RUST_VERSION=nightly-2019-08-26

# Install nightly rust
rustup toolchain install $RUST_VERSION
rustup target add i686-unknown-linux-gnu --toolchain $RUST_VERSION
rustup override set $RUST_VERSION

# Download dacapo
mkdir -p repos/jikesrvm/benchmarks
wget https://downloads.sourceforge.net/project/dacapobench/archive/2006-10-MR2/dacapo-2006-10-MR2.jar -O repos/jikesrvm/benchmarks/dacapo-2006-10-MR2.jar

# Install dependencies for JikesRVM
sudo apt-get update -y
sudo apt-get install build-essential gcc-multilib gettext bison -y

# Check toolchains' version
java -version
javac -version
echo $JAVA_HOME
cargo --version
rustup toolchain list
rustup target list