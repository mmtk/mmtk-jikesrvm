set -xe

. $(dirname "$0")/common.sh

# Install nightly rust
rustup toolchain install $RUSTUP_TOOLCHAIN
rustup target add i686-unknown-linux-gnu --toolchain $RUSTUP_TOOLCHAIN
rustup component add clippy --toolchain $RUSTUP_TOOLCHAIN
rustup component add rustfmt --toolchain $RUSTUP_TOOLCHAIN
rustup override set $RUSTUP_TOOLCHAIN

# Download dacapo
mkdir -p $DACAPO_PATH
wget https://downloads.sourceforge.net/project/dacapobench/archive/2006-10-MR2/dacapo-2006-10-MR2.jar -O $DACAPO_PATH/dacapo-2006-10-MR2.jar

# Install dependencies for JikesRVM
sudo apt-get update -y
sudo apt-get install build-essential gcc-multilib gettext bison -y
