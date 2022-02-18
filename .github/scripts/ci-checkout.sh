# ci-checkout.sh
set -ex

. $(dirname "$0")/common.sh

JIKESRVM_URL=`sed -n 's/^jikesrvm_repo.=."\(.*\)"$/\1/p' $BINDING_PATH/mmtk/Cargo.toml`
JIKESRVM_VERSION=`sed -n 's/^jikesrvm_version.=."\(.*\)"$/\1/p' $BINDING_PATH/mmtk/Cargo.toml`

rm -rf $JIKESRVM_PATH
git clone $JIKESRVM_URL $JIKESRVM_PATH
git -C $JIKESRVM_PATH checkout $JIKESRVM_VERSION
