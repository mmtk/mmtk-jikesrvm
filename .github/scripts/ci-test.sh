set -xe

# To JikesRVM folder
cd repos/jikesrvm

# Test BaseBase builds
# Only run one test

# BaseBaseNoGC
python scripts/testMMTk.py -j $JAVA_HOME -g BaseBaseNoGC -a "Xms1024M Xmx1024M" -- --use-third-party-heap=../../ --use-third-party-build-configs=../../jikesrvm/build/configs --use-external-source=../../jikesrvm/rvm/src -- --m32
# BaseBaseSemiSpace
python scripts/testMMTk.py -j $JAVA_HOME -g BaseBaseSemiSpace -a "Xms75M Xmx75M" -- --use-third-party-heap=../../ --use-third-party-build-configs=../../jikesrvm/build/configs --use-external-source=../../jikesrvm/rvm/src -- --m32

# Test FastAdaptive builds
# Run all possible dacapo benchmarks

# FastAdaptiveNoGC (use largest heap possible)
./bin/buildit localhost FastAdaptiveNoGC -j $JAVA_HOME --answer-yes --use-third-party-heap=../../ --use-third-party-build-configs=../../jikesrvm/build/configs --use-external-source=../../jikesrvm/rvm/src --m32
./dist/FastAdaptiveNoGC_x86_64_m32-linux/rvm -Xms3G -Xmx3G -jar benchmarks/dacapo-2006-10-MR2.jar antlr
./dist/FastAdaptiveNoGC_x86_64_m32-linux/rvm -Xms3G -Xmx3G -jar benchmarks/dacapo-2006-10-MR2.jar fop
./dist/FastAdaptiveNoGC_x86_64_m32-linux/rvm -Xms3G -Xmx3G -jar benchmarks/dacapo-2006-10-MR2.jar luindex
./dist/FastAdaptiveNoGC_x86_64_m32-linux/rvm -Xms3G -Xmx3G -jar benchmarks/dacapo-2006-10-MR2.jar pmd

RVM_OPTIONS='-X:gc:threads=16'

# RFastAdaptiveSemiSpace
./bin/buildit localhost FastAdaptiveSemiSpace -j $JAVA_HOME --answer-yes --use-third-party-heap=../../ --use-third-party-build-configs=../../jikesrvm/build/configs --use-external-source=../../jikesrvm/rvm/src --m32
./dist/FastAdaptiveSemiSpace_x86_64-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar antlr
./dist/FastAdaptiveSemiSpace_x86_64-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar bloat
./dist/FastAdaptiveSemiSpace_x86_64-linux/rvm $RVM_OPTIONS -Xms150M -Xmx150M -jar benchmarks/dacapo-2006-10-MR2.jar eclipse
./dist/FastAdaptiveSemiSpace_x86_64-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar fop
./dist/FastAdaptiveSemiSpace_x86_64-linux/rvm $RVM_OPTIONS -Xms200M -Xmx200M -jar benchmarks/dacapo-2006-10-MR2.jar hsqldb
./dist/FastAdaptiveSemiSpace_x86_64-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar jython
./dist/FastAdaptiveSemiSpace_x86_64-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar luindex
./dist/FastAdaptiveSemiSpace_x86_64-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar lusearch
./dist/FastAdaptiveSemiSpace_x86_64-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar pmd
./dist/FastAdaptiveSemiSpace_x86_64-linux/rvm $RVM_OPTIONS -Xms100M -Xmx100M -jar benchmarks/dacapo-2006-10-MR2.jar xalan

# RFastAdaptiveMarkSweep
./bin/buildit localhost FastAdaptiveMarkSweep -j $JAVA_HOME --answer-yes --use-third-party-heap=../../ --use-third-party-build-configs=../../jikesrvm/build/configs --use-external-source=../../jikesrvm/rvm/src --m32
./dist/FastAdaptiveMarkSweep_x86_64-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar bloat
# Failing instruction offset: 0x000000c3 in method ___ with descriptor ___ Couldn't find a method for given instruction offset
#./dist/RFastAdaptiveMarkSweep_x86_64-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar antlr - fail non-deterministically, basebase build runs fine with assertions
#./dist/RFastAdaptiveMarkSweep_x86_64-linux/rvm $RVM_OPTIONS -Xms150M -Xmx150M -jar benchmarks/dacapo-2006-10-MR2.jar eclipse
./dist/FastAdaptiveMarkSweep_x86_64-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar fop
./dist/FastAdaptiveMarkSweep_x86_64-linux/rvm $RVM_OPTIONS -Xms200M -Xmx200M -jar benchmarks/dacapo-2006-10-MR2.jar hsqldb
./dist/FastAdaptiveMarkSweep_x86_64-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar jython
./dist/FastAdaptiveMarkSweep_x86_64-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar luindex
./dist/FastAdaptiveMarkSweep_x86_64-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar lusearch
./dist/FastAdaptiveMarkSweep_x86_64-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar pmd
./dist/FastAdaptiveMarkSweep_x86_64-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar xalan
