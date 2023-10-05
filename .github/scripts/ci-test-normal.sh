set -xe

. $(dirname "$0")/common.sh

# To JikesRVM folder
cd $JIKESRVM_PATH

# Test BaseBase builds
# Only run one test

# RBaseBaseNoGC
./bin/buildit localhost RBaseBaseNoGC -j $JAVA_HOME --answer-yes --use-third-party-heap=$BINDING_PATH --use-third-party-build-configs=$BINDING_PATH/jikesrvm/build/configs --use-external-source=$BINDING_PATH/jikesrvm/rvm/src --m32
./dist/RBaseBaseNoGC_x86_64_m32-linux/rvm -Xmx1224M -Xms1024M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar fop
# RBaseBaseSemiSpace
./bin/buildit localhost RBaseBaseSemiSpace -j $JAVA_HOME --answer-yes --use-third-party-heap=$BINDING_PATH --use-third-party-build-configs=$BINDING_PATH/jikesrvm/build/configs --use-external-source=$BINDING_PATH/jikesrvm/rvm/src --m32
./dist/RBaseBaseSemiSpace_x86_64_m32-linux/rvm -Xmx400M -Xms75M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar fop

# Test FastAdaptive builds
# Run all possible dacapo benchmarks

# RFastAdaptiveNoGC (use largest heap possible)
./bin/buildit localhost RFastAdaptiveNoGC -j $JAVA_HOME --answer-yes --use-third-party-heap=$BINDING_PATH --use-third-party-build-configs=$BINDING_PATH/jikesrvm/build/configs --use-external-source=$BINDING_PATH/jikesrvm/rvm/src --m32
./dist/RFastAdaptiveNoGC_x86_64_m32-linux/rvm -Xms3G -Xmx3G -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar antlr
./dist/RFastAdaptiveNoGC_x86_64_m32-linux/rvm -Xms3G -Xmx3G -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar fop
./dist/RFastAdaptiveNoGC_x86_64_m32-linux/rvm -Xms3G -Xmx3G -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar luindex
./dist/RFastAdaptiveNoGC_x86_64_m32-linux/rvm -Xms3G -Xmx3G -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar pmd

export MMTK_THREADS=16

# RFastAdaptiveSemiSpace
./bin/buildit localhost RFastAdaptiveSemiSpace -j $JAVA_HOME --answer-yes --use-third-party-heap=$BINDING_PATH/ --use-third-party-build-configs=$BINDING_PATH/jikesrvm/build/configs --use-external-source=$BINDING_PATH/jikesrvm/rvm/src --m32
./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx600M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar antlr
./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx1900M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar bloat
#fail ./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx150M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar eclipse
./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx400M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar fop
#fail ./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms200M -Xmx200M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar hsqldb
./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx2000M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar jython
./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx900M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar luindex
#fail./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar lusearch
./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx1500M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar pmd
./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms100M -Xmx1900M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar xalan

# RFastAdaptiveMarkSweep
./bin/buildit localhost RFastAdaptiveMarkSweep -j $JAVA_HOME --answer-yes --use-third-party-heap=$BINDING_PATH/ --use-third-party-build-configs=$BINDING_PATH/jikesrvm/build/configs --use-external-source=$BINDING_PATH/jikesrvm/rvm/src --m32
./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx400M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar antlr

# Flaky test: Failing instruction starting at xxxxx wasn't in RVM address space
# see https://github.com/mmtk/mmtk-jikesrvm/issues/108
#./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx150M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar bloat

# Failing instruction offset: 0x000000c3 in method ___ with descriptor ___ Couldn't find a method for given instruction offset
#./dist/RFastAdaptiveMarkSweep_x86_64-linux/rvm $RVM_OPTIONS -Xms150M -Xmx150M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar eclipse
./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx350M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar fop
#fail./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms300M -Xmx300M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar hsqldb
./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx1100M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar jython
./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx650M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar luindex
#fail./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx150M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar lusearch
./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx800M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar pmd
./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx900M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar xalan
