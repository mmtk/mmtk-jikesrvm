set -xe

# To JikesRVM folder
root_dir=$(dirname "$0")/../../
cd $root_dir/repos/jikesrvm

# Test BaseBase builds
# Only run one test

# RBaseBaseNoGC
python scripts/testMMTk.py -j $JAVA_HOME -g RBaseBaseNoGC -a "Xms1024M Xmx1024M" -- --use-third-party-heap=../../ --use-third-party-build-configs=../../jikesrvm/build/configs --use-external-source=../../jikesrvm/rvm/src --m32
# RBaseBaseSemiSpace
python scripts/testMMTk.py -j $JAVA_HOME -g RBaseBaseSemiSpace -a "Xms75M Xmx75M" -- --use-third-party-heap=../../ --use-third-party-build-configs=../../jikesrvm/build/configs --use-external-source=../../jikesrvm/rvm/src --m32

# Test FastAdaptive builds
# Run all possible dacapo benchmarks

# RFastAdaptiveNoGC (use largest heap possible)
./bin/buildit localhost RFastAdaptiveNoGC -j $JAVA_HOME --answer-yes --use-third-party-heap=../../ --use-third-party-build-configs=../../jikesrvm/build/configs --use-external-source=../../jikesrvm/rvm/src --m32
./dist/RFastAdaptiveNoGC_x86_64_m32-linux/rvm -Xms3G -Xmx3G -jar benchmarks/dacapo-2006-10-MR2.jar antlr
./dist/RFastAdaptiveNoGC_x86_64_m32-linux/rvm -Xms3G -Xmx3G -jar benchmarks/dacapo-2006-10-MR2.jar fop
./dist/RFastAdaptiveNoGC_x86_64_m32-linux/rvm -Xms3G -Xmx3G -jar benchmarks/dacapo-2006-10-MR2.jar luindex
./dist/RFastAdaptiveNoGC_x86_64_m32-linux/rvm -Xms3G -Xmx3G -jar benchmarks/dacapo-2006-10-MR2.jar pmd

RVM_OPTIONS='-X:gc:threads=16'

# RFastAdaptiveSemiSpace
./bin/buildit localhost RFastAdaptiveSemiSpace -j $JAVA_HOME --answer-yes --use-third-party-heap=../../ --use-third-party-build-configs=../../jikesrvm/build/configs --use-external-source=../../jikesrvm/rvm/src --m32
./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar antlr
./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar bloat
./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms150M -Xmx150M -jar benchmarks/dacapo-2006-10-MR2.jar eclipse
./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar fop
./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms200M -Xmx200M -jar benchmarks/dacapo-2006-10-MR2.jar hsqldb
./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar jython
./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar luindex
./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar lusearch
./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar pmd
./dist/RFastAdaptiveSemiSpace_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms100M -Xmx100M -jar benchmarks/dacapo-2006-10-MR2.jar xalan

# RFastAdaptiveMarkSweep
./bin/buildit localhost RFastAdaptiveMarkSweep -j $JAVA_HOME --answer-yes --use-third-party-heap=../../ --use-third-party-build-configs=../../jikesrvm/build/configs --use-external-source=../../jikesrvm/rvm/src --m32
./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar bloat
# Failing instruction offset: 0x000000c3 in method ___ with descriptor ___ Couldn't find a method for given instruction offset
#./dist/RFastAdaptiveMarkSweep_x86_64-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar antlr - fail non-deterministically, basebase build runs fine with assertions
#./dist/RFastAdaptiveMarkSweep_x86_64-linux/rvm $RVM_OPTIONS -Xms150M -Xmx150M -jar benchmarks/dacapo-2006-10-MR2.jar eclipse
./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar fop
./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms200M -Xmx200M -jar benchmarks/dacapo-2006-10-MR2.jar hsqldb
./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar jython
./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar luindex
./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar lusearch
./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar pmd
./dist/RFastAdaptiveMarkSweep_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar xalan
