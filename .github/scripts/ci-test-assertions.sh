set -xe

# To JikesRVM folder
cd $JIKESRVM_PATH

# RBaseBaseSemiSpaceAssertions
./bin/buildit localhost RBaseBaseSemiSpaceAssertions -j $JAVA_HOME --answer-yes --use-third-party-heap=$BINDING_PATH --use-third-party-build-configs=$BINDING_PATH/jikesrvm/build/configs --use-external-source=$BINDING_PATH/jikesrvm/rvm/src --m32
./dist/RBaseBaseSemiSpaceAssertions_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar fop
./dist/RBaseBaseSemiSpaceAssertions_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar luindex

# RBaseBaseMarkSweepAssertions
./bin/buildit localhost RBaseBaseMarkSweepAssertions -j $JAVA_HOME --answer-yes --use-third-party-heap=$BINDING_PATH --use-third-party-build-configs=$BINDING_PATH/jikesrvm/build/configs --use-external-source=$BINDING_PATH/jikesrvm/rvm/src --m32
./dist/RBaseBaseMarkSweepAssertions_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar fop
./dist/RBaseBaseMarkSweepAssertions_x86_64_m32-linux/rvm $RVM_OPTIONS -Xms75M -Xmx75M -jar $DACAPO_PATH/dacapo-2006-10-MR2.jar luindex
