# MMTk-JikesRVM  
  
This repository provides the JikesRVM binding for MMTk.
  
## Contents

* [Requirements](#requirements)
* [Build](#build)
* [Test](#test)
  
## Requirements
  
We maintain an up to date list of the prerequisites for building MMTk and its bindings in the [mmtk-dev](https://github.com/mmtk/mmtk-dev) repository.
Please make sure your dev machine satisfies those prerequisites.

MMTk/JikesRVM supports `linux-i686` and `linux-x86_64` (as a 32-bit program).
  
### Before you continue

If you use the set-up explained in [mmtk-dev](https://github.com/mmtk/mmtk-dev), make sure to do the following steps before continuing to the [Build](#build) section:

1. Set the default Rust toolchain to the one specified in [mmtk-dev](https://github.com/mmtk/mmtk-dev), e.g. by running:

```console
$ # replace nightly-YYYY-MM-DD with the toolchain specified in mmtk-dev
$ Export RUSTUP_TOOLCHAIN=nightly-YYYY-MM-DD
```

2. Set `openjdk-8-jdk` as the default JDK (openjdk-8-jdk is a build requirement of JikesRVM), e.g. by running:

```console
$ update-java-alternatives --set java-1.8.0-openjdk-amd64
```

3. You may also need to use ssh-agent to authenticate with github (see [here](https://github.com/rust-lang/cargo/issues/3487) for more info):

```console
$ eval `ssh-agent`
$ ssh-add
```

### Getting Sources (for MMTk and JikesRVM)

You will need the correct revisions of MMTk and JikesRVM.
Both are checked in as git submodules under `repos`.
You would simply need to run the following lines under the root directory of `mmtk-jikesrvm` to fetch submodules' sources for MMTk and JikesRVM:

```console
$ git submodule init
$ git submodule update
```

Alternatively, you could fetch the sources by yourself (make sure you have the right VM/MMTk revisions that match the `mmtk-jikesrvm` revision.
If you clone the MMTk core in a folder other than `repos/mmtk-core`, you would need to modify `mmtk/Cargo.toml` to point the `mmtk` dependency to your MMTk core folder.

* JikesRVM: [https://github.com/mmtk/jikesrvm](https://github.com/mmtk/jikesrvm)
* MMTk Core: [https://github.com/mmtk/mmtk-core](https://github.com/mmtk/mmtk-core)

The rest of this instruction assumes you have done the `git submodule init/update` and have both repositories under `repos`.

## Build

MMTk building is integrated as as a step of the JikesRVM build.
We recommend using the `buildit` script for the JikesRVM build.

```console
$ cd repos/jikesrvm
$ ./bin/buildit localhost RBaseBaseSemiSpace --use-third-party-heap=../../ --use-third-party-build-configs=../../jikesrvm/build/configs/ --use-external-source=../../jikesrvm/rvm/src
```

The JikesRVM binary is under `repos/jikesrvm/dist/RBaseBaseSemiSpace_x86_64-linux/rvm` and the MMTk shared library is `repos/jikesrvm/dist/RBaseBaseSemiSpace_x86_64-linux/libmmtk.so`.

You can build with other build configs, check `jikesrvm/build/configs`.

## Test

### Run DaCapo Benchmarks

Fetch DaCapo:

```console
$ # run from the root repo directory
$ mkdir -p repos/jikesrvm/benchmarks
$ wget https://downloads.sourceforge.net/project/dacapobench/archive/2006-10-MR2/dacapo-2006-10-MR2.jar -O repos/jikesrvm/benchmarks/dacapo-2006-10-MR2.jar
```

Run `rvm`:

```console
$ LD_LIBRARY_PATH=repos/jikesrvm/dist/RBaseBaseSemiSpace_x86_64-linux/ repos/jikesrvm/dist/RBaseBaseSemiSpace_x86_64-linux/rvm -Xms75M -Xmx75M -jar repos/jikesrvm/benchmarks/dacapo-2006-10-MR2.jar fop
===== DaCapo fop starting =====
ThreadId(1)[INFO:/root/mmtk-jikesrvm/repos/mmtk-core/src/plan/global.rs:112]   [POLL] copyspace0: Triggering collection
ThreadId(1)[INFO:/root/mmtk-jikesrvm/repos/mmtk-core/src/plan/global.rs:112]   [POLL] copyspace1: Triggering collection
ThreadId(1)[INFO:/root/mmtk-jikesrvm/repos/mmtk-core/src/plan/global.rs:112]   [POLL] copyspace0: Triggering collection
ThreadId(1)[INFO:/root/mmtk-jikesrvm/repos/mmtk-core/src/plan/global.rs:112]   [POLL] immortal: Triggering collection
ThreadId(1)[INFO:/root/mmtk-jikesrvm/repos/mmtk-core/src/plan/global.rs:112]   [POLL] copyspace0: Triggering collection
ThreadId(1)[INFO:/root/mmtk-jikesrvm/repos/mmtk-core/src/plan/global.rs:112]   [POLL] copyspace1: Triggering collection
===== DaCapo fop PASSED in 3934 msec =====
```
