set -e

. $(dirname "$0")/common.sh

cur=$(realpath $(dirname "$0"))

cd $cur
./ci-test-normal.sh
cd $cur
./ci-test-assertions.sh
cd $cur
./ci-test-weak-ref.sh
