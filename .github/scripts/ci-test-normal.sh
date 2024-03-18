set -xe

. $(dirname "$0")/common.sh

# To JikesRVM folder
cd $JIKESRVM_PATH

# Test BaseBase builds
# Only run one test

# RBaseBaseNoGC
./bin/buildit localhost RBaseBaseNoGC -j $JAVA_HOME --answer-yes --use-third-party-heap=$BINDING_PATH --use-third-party-build-configs=$BINDING_PATH/jikesrvm/build/configs --use-external-source=$BINDING_PATH/jikesrvm/rvm/src --m32
setarch -R ./dist/RBaseBaseNoGC_x86_64_m32-linux/rvm -Xmx1024M -Xms1024M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar fop
# RBaseBaseSemiSpace
./bin/buildit localhost RBaseBaseSemiSpace -j $JAVA_HOME --answer-yes --use-third-party-heap=$BINDING_PATH --use-third-party-build-configs=$BINDING_PATH/jikesrvm/build/configs --use-external-source=$BINDING_PATH/jikesrvm/rvm/src --m32
setarch -R ./dist/RBaseBaseSemiSpace_x86_64_m32-linux/rvm -Xmx75M -Xms75M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar fop

# Test FastAdaptive builds
# Run all possible dacapo benchmarks

# RFastAdaptiveNoGC (use largest heap possible)
./bin/buildit localhost RFastAdaptiveNoGC -j $JAVA_HOME --answer-yes --use-third-party-heap=$BINDING_PATH --use-third-party-build-configs=$BINDING_PATH/jikesrvm/build/configs --use-external-source=$BINDING_PATH/jikesrvm/rvm/src --m32
setarch -R ./dist/RFastAdaptiveNoGC_x86_64_m32-linux/rvm -Xms3G -Xmx3G -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar antlr
setarch -R ./dist/RFastAdaptiveNoGC_x86_64_m32-linux/rvm -Xms3G -Xmx3G -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar fop
setarch -R ./dist/RFastAdaptiveNoGC_x86_64_m32-linux/rvm -Xms3G -Xmx3G -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar luindex
setarch -R ./dist/RFastAdaptiveNoGC_x86_64_m32-linux/rvm -Xms3G -Xmx3G -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar pmd

export MMTK_THREADS=16

# RFastAdaptiveSemiSpace
./bin/buildit localhost RFastAdaptiveSemiSpace -j $JAVA_HOME --answer-yes --use-third-party-heap=$BINDING_PATH/ --use-third-party-build-configs=$BINDING_PATH/jikesrvm/build/configs --use-external-source=$BINDING_PATH/jikesrvm/rvm/src --m32
setarch -R ./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar antlr
setarch -R ./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar bloat
setarch -R ./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx150M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar eclipse
setarch -R ./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar fop
setarch -R ./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms200M -Xmx200M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar hsqldb
setarch -R ./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar jython
setarch -R ./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar luindex
setarch -R ./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar lusearch
setarch -R ./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar pmd
setarch -R ./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms100M -Xmx100M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar xalan

# RFastAdaptiveMarkSweep
./bin/buildit localhost RFastAdaptiveMarkSweep -j $JAVA_HOME --answer-yes --use-third-party-heap=$BINDING_PATH/ --use-third-party-build-configs=$BINDING_PATH/jikesrvm/build/configs --use-external-source=$BINDING_PATH/jikesrvm/rvm/src --m32
setarch -R ./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx150M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar antlr
setarch -R ./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx150M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar bloat
# Failing instruction offset: 0x000000c3 in method ___ with descriptor ___ Couldn't find a method for given instruction offset
# setarch -R ./dist/RFastAdaptiveMarkSweep_x86_64-linux/rvm $RVM_OPTIONS -Xms150M -Xmx150M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar eclipse
setarch -R ./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx150M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar fop
setarch -R ./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms300M -Xmx300M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar hsqldb
setarch -R ./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx150M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar jython
setarch -R ./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx150M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar luindex
setarch -R ./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx150M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar lusearch
setarch -R ./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx150M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar pmd
setarch -R ./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx150M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar xalan
