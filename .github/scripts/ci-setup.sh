set -xe

# Install nightly rust
rustup toolchain install $RUSTUP_TOOLCHAIN
rustup target add i686-unknown-linux-gnu --toolchain $RUSTUP_TOOLCHAIN
rustup component add clippy --toolchain $RUSTUP_TOOLCHAIN
rustup component add rustfmt --toolchain $RUSTUP_TOOLCHAIN
rustup override set $RUSTUP_TOOLCHAIN

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