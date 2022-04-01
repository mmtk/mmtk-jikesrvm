# MMTk-JikesRVM  
  
This repository provides the JikesRVM binding for MMTk.
  
## Contents

* [Requirements](#requirements)
* [Build](#build)
* [Test](#test)
  
## Requirements
  
We maintain an up to date list of the prerequisites for building MMTk and its bindings in the [mmtk-dev-env](https://github.com/mmtk/mmtk-dev-env) repository.
Please make sure your dev machine satisfies those prerequisites.

MMTk/JikesRVM supports `linux-i686` and `linux-x86_64` (as a 32-bit program).
  
### Before you continue

If you use the set-up explained in [mmtk-dev-env](https://github.com/mmtk/mmtk-dev-env), make sure to do the following steps before continuing to the [Build](#build) section:

1. Use a proper Rust toolchain. The minimal supported Rust version for MMTk-JikesRVM binding is 1.xx.0. Make sure your Rust version is higher than this. We test MMTk-JikesRVM
binding with Rust 1.59.0 (as specified in [`rust-toolchain`](mmtk/rust-toolchain)).

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

To work on JikesRVM binding, we expect you have a directory structure like below. This section gives instructions on how to check out
those repositories with the correct version.

```
Your working directory/
├─ mmtk-jikesrvm/
│  ├─ jikesrvm/
│  └─ mmtk/
├─ jikesrvm/
└─ mmtk-core/ (optional)
```

#### Checkout Binding

First clone this binding repo:

```console
$ git clone https://github.com/mmtk/mmtk-jikesrvm.git
```

The binding repo mainly consists of two folders, `mmtk` and `jikesrvm`.
* `mmtk` is logically a part of MMTk. It exposes APIs from `mmtk-core` and implements the `VMBinding` trait from `mmtk-core`.
* `jikesrvm` is logically a part of JikesRVM. When we build JikesRVM, we copy this folder to the JikesRVM repo (which overwrite the same files,
  if any, in the JikesRVM repo) and treat it as if it is a part of the JikesRVM project.

#### Checkout JikesRVM

You would need our JikesRVM fork which includes the support for a third party heap (like MMTk). We assume you put `jikesrvm` as a sibling of `mmtk-jikesrvm`.
[`Cargo.toml`](mmtk/Cargo.toml) defines the version of JikesRVM that works with the version of `mmtk-jikesrvm`.

Assuming your current working directory is the parent folder of `mmtk-jikesrvm`, you can checkout out JikesRVM and the correct version using:
```console
$ git clone https://github.com/mmtk/jikesrvm.git
$ git -C jikesrvm checkout `sed -n 's/^jikesrvm_version.=."\(.*\)"$/\1/p' < mmtk-jikesrvm/mmtk/Cargo.toml`
```

#### Checkout MMTk core (optional)

The MMTk-JikesRVM binding points to a specific version of `mmtk-core` as defined in [`Cargo.toml`](mmtk/Cargo.toml). When you build the binding,
cargo will fetch the specified version of `mmtk-core`. If you would like to use
a different version or a local `mmtk-core` repo, you can checkout `mmtk-core` to a separate repo and modify the `mmtk` dependency in `Cargo.toml`.

For example, you can check out `mmtk-core` as a sibling of `mmtk-jikesrvm`.

```console
$ git clone https://github.com/mmtk/mmtk-core.git
```

And change the `mmtk` dependency in `Cargo.toml` (this assumes you put `mmtk-core` as a sibling of `mmtk-jikesrvm`):

```toml
mmtk = { path = "../../mmtk-core" }
```

## Build

MMTk building is integrated as as a step of the JikesRVM build.
We recommend using the `buildit` script for the JikesRVM build.

```console
$ cd repos/jikesrvm
$ ./bin/buildit localhost RBaseBaseSemiSpace --use-third-party-heap=../mmtk-jikesrvm --use-third-party-build-configs=../mmtk-jikesrvm/jikesrvm/build/configs/ --use-external-source=../mmtk-jikesrvm/jikesrvm/rvm/src --m32
```

The JikesRVM binary is under `jikesrvm/dist/RBaseBaseSemiSpace_x86_64_m32-linux/rvm` and the MMTk shared library is `jikesrvm/dist/RBaseBaseSemiSpace_x86_64_m32-linux/libmmtk.so`.

You can build with other build configs, check `mmtk-jikesrvm/jikesrvm/build/configs`.

## Test

### Run DaCapo Benchmarks

Fetch DaCapo:

```console
$ # run from the repo/jikesrvm directory
$ mkdir -p benchmarks
$ wget https://downloads.sourceforge.net/project/dacapobench/archive/2006-10-MR2/dacapo-2006-10-MR2.jar -O benchmarks/dacapo-2006-10-MR2.jar
```

Run `rvm`:

```console
$ LD_LIBRARY_PATH=dist/RBaseBaseSemiSpace_x86_64_m32-linux/ dist/RBaseBaseSemiSpace_x86_64_m32-linux/rvm -Xms75M -Xmx75M -jar benchmarks/dacapo-2006-10-MR2.jar fop
===== DaCapo fop starting =====
ThreadId(1)[INFO:/root/mmtk-core/src/plan/global.rs:112]   [POLL] copyspace0: Triggering collection
ThreadId(1)[INFO:/root/mmtk-core/src/plan/global.rs:112]   [POLL] copyspace1: Triggering collection
ThreadId(1)[INFO:/root/mmtk-core/src/plan/global.rs:112]   [POLL] copyspace0: Triggering collection
ThreadId(1)[INFO:/root/mmtk-core/src/plan/global.rs:112]   [POLL] immortal: Triggering collection
ThreadId(1)[INFO:/root/mmtk-core/src/plan/global.rs:112]   [POLL] copyspace0: Triggering collection
ThreadId(1)[INFO:/root/mmtk-core/src/plan/global.rs:112]   [POLL] copyspace1: Triggering collection
===== DaCapo fop PASSED in 3934 msec =====
```
