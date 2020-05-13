# mmtk-jikesrvm  
  
This repository provides the JikesRVM binding for MMTk.  
  
## Contents  
* [Requirements](#requirements)  
* [Build](#build)  
* [Test](#test)  
  
## Requirements  
  
This section describes prerequisite for building JikesRVM with MMTk.  
  
### Before You Start  
  
#### Software Dependencies  
  
* JikesRVM  
  * Please check [tool requirements for JikesRVM](https://www.jikesrvm.org/UserGuide/BuildingJikesRVM/index.html#x5-160003.3)  
  * You need to use [our JikesRVM fork](https://gitlab.anu.edu.au/mmtk/jikesrvm), which allows building JikesRVM with a third party memory manager.  
* MMTk  
  * MMTk requires the rustup nightly toolchain. 
  * Please visit [rustup.rs](https://rustup.rs/) for installation instructions.
  * We are testing with Rust 1.39.0 nightly. Theoretically it should work with any Rust nightly version higher than 1.39.0. However, if you encounter any issue with any newer Rust version, please submit an issue to let us know. 

#### Supported Hardware

MMTk/JikesRVM supports `linux-i686` and `linux-x86_64` (as 32bits program).
Tested on Ubuntu 18.04.4 LTS (GNU/Linux 4.15.0-21-generic x86_64). 

### Getting Sources (for MMTk and JikesRVM)

You would need the correct revisions of MMTk and JikesRVM. Both are checked in as git submodules under `repos`. You would simply need to run the following lines under the root directory of `mmtk-jikesrvm` to fetch submodules' sources for MMTk and JikesRVM:
```
git submodule init
git submodule update
```
Alternatively, you could fetch the sources by yourself (make sure you have the right VM/MMTk revisions that match the `mmtk-jikesrvm` revision. If you clone the MMTk core in a folder other than `repos/mmtk-core`, you would need to modify `mmtk/Cargo.toml` to point the `mmtk` dependency to your MMTk core folder.
* JikesRVM: [https://github.com/mmtk/jikesrvm](https://github.com/mmtk/jikesrvm)
* MMTk Core: [https://github.com/mmtk/mmtk-core](https://github.com/mmtk/mmtk-core)

The rest of this instruction assumes you have done the `git submodule init/update` and have both repositories under `repos`.

## Build

MMTk building is integrated as one step during JikesRVM build. We recommend using the `buildit` script for the JikesRVM build.
```
cd repos/jikesrvm
./bin/buildit localhost RBaseBaseSemiSpace --use-third-party-heap=../../ --use-third-party-build-configs=../../jikesrvm/build/configs/ --use-external-source=../../jikesrvm/rvm/src
```

The JikesRVM binary is under `repos/jikesrvm/dist/RBaseBaseSemiSpace_x86_64-linux/rvm` and the MMTk shared library is `repos/jikesrvm/dist/RBaseBaseSemiSpace_x86_64-linux/libmmtk.so`. 

You could build with other build configs, check `jikesrvm/build/configs`.

## Test

### Run DaCapo Benchmarks

Fetch DaCapo:
```
mkdir -p repos/jikesrvm/benchmarks
wget https://downloads.sourceforge.net/project/dacapobench/archive/2006-10-MR2/dacapo-2006-10-MR2.jar -O repos/jikesrvm/benchmarks/dacapo-2006-10-MR2.jar
```

Run `rvm`:
```console
yilin@elk:~/RustWorkspace/mmtk/jikesrvm-binding$ LD_LIBRARY_PATH=repos/jikesrvm/dist/RBaseBaseSemiSpace_x86_64-linux/ repos/jikesrvm/dist/RBaseBaseSemiSpace_x86_64-linux/rvm -Xms75M -Xmx75M -jar repos/jikesrvm/benchmarks/dacapo-2006-10-MR2.jar fop
===== DaCapo fop starting =====
ThreadId(1)[INFO:/home/yilin/RustWorkspace/mmtk/jikesrvm-binding/repos/mmtk-core/src/plan/plan.rs:96]   [POLL] copyspace0: Triggering collection
ThreadId(1)[INFO:/home/yilin/RustWorkspace/mmtk/jikesrvm-binding/repos/mmtk-core/src/plan/plan.rs:96]   [POLL] copyspace1: Triggering collection
ThreadId(1)[INFO:/home/yilin/RustWorkspace/mmtk/jikesrvm-binding/repos/mmtk-core/src/plan/plan.rs:96]   [POLL] copyspace0: Triggering collection
ThreadId(1)[INFO:/home/yilin/RustWorkspace/mmtk/jikesrvm-binding/repos/mmtk-core/src/plan/plan.rs:96]   [POLL] copyspace1: Triggering collection
ThreadId(1)[INFO:/home/yilin/RustWorkspace/mmtk/jikesrvm-binding/repos/mmtk-core/src/plan/plan.rs:96]   [POLL] copyspace0: Triggering collection
ThreadId(1)[INFO:/home/yilin/RustWorkspace/mmtk/jikesrvm-binding/repos/mmtk-core/src/plan/plan.rs:96]   [POLL] copyspace1: Triggering collection
===== DaCapo fop PASSED in 3692 msec =====
```
