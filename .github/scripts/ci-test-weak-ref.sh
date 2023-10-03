set -xe

. $(dirname "$0")/common.sh

# To JikesRVM folder
cd $JIKESRVM_PATH

# Test FastAdaptive builds
# Run all possible dacapo benchmarks

export MMTK_THREADS=16
export RUST_BACKTRACE=1
RVM_OPTIONS=-X:gc:no_reference_types=false

# RFastAdaptiveSemiSpace
./bin/buildit localhost RFastAdaptiveSemiSpace -j $JAVA_HOME --answer-yes --use-third-party-heap=$BINDING_PATH/ --use-third-party-build-configs=$BINDING_PATH/jikesrvm/build/configs --use-external-source=$BINDING_PATH/jikesrvm/rvm/src --m32
#fail ./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx150M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar eclipse
./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx400M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar antlr
#fail ./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx750M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar bloat
./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx200M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar fop
#fail ./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms200M -Xmx200M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar hsqldb
./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx700M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar luindex
./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx1700M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar jython
#fail ./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx750M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar lusearch
./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx900M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar pmd
./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms100M -Xmx1500M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar xalan

# RFastAdaptiveMarkSweep
./bin/buildit localhost RFastAdaptiveMarkSweep -j $JAVA_HOME --answer-yes --use-third-party-heap=$BINDING_PATH/ --use-third-party-build-configs=$BINDING_PATH/jikesrvm/build/configs --use-external-source=$BINDING_PATH/jikesrvm/rvm/src --m32
./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx200M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar antlr
./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx900M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar bloat
# Failing instruction offset: 0x000000c3 in method ___ with descriptor ___ Couldn't find a method for given instruction offset
# ./dist/RFastAdaptiveMarkSweep_x86_64-linux/rvm $RVM_OPTIONS -Xms150M -Xmx150M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar eclipse
./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx150M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar fop
#fail ./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms300M -Xmx300M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar hsqldb
./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx900M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar jython
./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx400M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar luindex
#fail ./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx150M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar lusearch
./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx600M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar pmd
./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx750M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar xalan
