set -xe

# To JikesRVM folder
root_dir=$(dirname "$0")../../
cd $root_dir/repos/jikesrvm

# RBaseBaseSemiSpaceAssertions
./bin/buildit localhost RBaseBaseSemiSpaceAssertions -j $JAVA_HOME --answer-yes --use-third-party-heap=../../ --use-third-party-build-configs=../../jikesrvm/build/configs --use-external-source=../../jikesrvm/rvm/src --m32
./dist/RBaseBaseSemiSpaceAssertions_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar fop
./dist/RBaseBaseSemiSpaceAssertions_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar luindex

# RBaseBaseMarkSweepAssertions
./bin/buildit localhost RBaseBaseMarkSweepAssertions -j $JAVA_HOME --answer-yes --use-third-party-heap=../../ --use-third-party-build-configs=../../jikesrvm/build/configs --use-external-source=../../jikesrvm/rvm/src --m32
./dist/RBaseBaseMarkSweepAssertions_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar fop
./dist/RBaseBaseMarkSweepAssertions_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar luindex
