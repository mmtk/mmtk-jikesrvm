set -xe

. $(dirname "$0")/common.sh

# To JikesRVM folder
cd $JIKESRVM_PATH

# Test FastAdaptive builds
# Run all possible Dacapo benchmarks

export MMTK_THREADS=16
export RUST_BACKTRACE=1
RVM_OPTIONS=-X:gc:no_reference_types=false

# Directory containing properties files
PROPERTIES_DIR="$BINDING_PATH/jikesrvm/build/configs"

# Find all .properties files and update them using sed
find "$PROPERTIES_DIR" -type f -name "*.properties" -exec sed -i 's/rust.binding_side_ref_proc=false/rust.binding_side_ref_proc=true/' {} \;


# RFastAdaptiveSemiSpace
setarch -R ./bin/buildit localhost RFastAdaptiveSemiSpace -j $JAVA_HOME --answer-yes --use-third-party-heap=$BINDING_PATH/ --use-third-party-build-configs=$BINDING_PATH/jikesrvm/build/configs --use-external-source=$BINDING_PATH/jikesrvm/rvm/src --m32
#fail setarch -R ./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx150M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar eclipse
setarch -R ./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms600M -Xmx600M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar antlr
setarch -R ./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms2G -Xmx2G -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar bloat
setarch -R ./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms400M -Xmx400M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar fop
#fail setarch -R ./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms200M -Xmx200M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar hsqldb
setarch -R ./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms900M -Xmx900M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar luindex
setarch -R ./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms1900M -Xmx1900M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar jython
#fail setarch -R ./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar lusearch
setarch -R ./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms1500M -Xmx1500M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar pmd
setarch -R ./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms1800M -Xmx1800M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar xalan

# RFastAdaptiveMarkSweep
./bin/buildit localhost RFastAdaptiveMarkSweep -j $JAVA_HOME --answer-yes --use-third-party-heap=$BINDING_PATH/ --use-third-party-build-configs=$BINDING_PATH/jikesrvm/build/configs --use-external-source=$BINDING_PATH/jikesrvm/rvm/src --m32
setarch -R ./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms400M -Xmx400M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar antlr

# Flaky test: Failing instruction starting at xxxxx wasn't in RVM address space
# see https://github.com/mmtk/mmtk-jikesrvm/issues/108
# setarch -R ./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx150M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar bloat

# Failing instruction offset: 0x000000c3 in method ___ with descriptor ___ Couldn't find a method for given instruction offset
# setarch -R ./dist/RFastAdaptiveMarkSweep_x86_64-linux/rvm $RVM_OPTIONS -Xms150M -Xmx150M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar eclipse
setarch -R ./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms350M -Xmx350M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar fop
#fail setarch -R ./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms300M -Xmx300M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar hsqldb
setarch -R ./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms1100M -Xmx1100M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar jython
setarch -R ./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms600M -Xmx600M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar luindex
#fail setarch -R ./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx150M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar lusearch
setarch -R ./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms800M -Xmx800M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar pmd
setarch -R ./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms950M -Xmx950M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar xalan

# Find all .properties files and update them using sed
find "$PROPERTIES_DIR" -type f -name "*.properties" -exec sed -i 's/rust.binding_side_ref_proc=true/rust.binding_side_ref_proc=false/' {} \;
