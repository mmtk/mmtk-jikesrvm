0.19.0 (2023-08-18)
===

## What's Changed
* Set VM space start and size through options by @qinsoon in https://github.com/mmtk/mmtk-jikesrvm/pull/139
* Install the missing deps in CI tests by @qinsoon in https://github.com/mmtk/mmtk-jikesrvm/pull/140
* Update to MMTk core PR #817 by @qinsoon in https://github.com/mmtk/mmtk-jikesrvm/pull/141
* Update to mmtk-core PR #838 by @qinsoon in https://github.com/mmtk/mmtk-jikesrvm/pull/142
* Rename ambiguous `scan_thread_root{,s}` functions by @k-sareen in https://github.com/mmtk/mmtk-jikesrvm/pull/143
* Set `REFS` to 0 when starting the scan boot image by @k-sareen in https://github.com/mmtk/mmtk-jikesrvm/pull/144
* Add set -e in ci-test.sh. Update MMTk by @qinsoon in https://github.com/mmtk/mmtk-jikesrvm/pull/146
* Update to MMTk core PR #875 by @qinsoon in https://github.com/mmtk/mmtk-jikesrvm/pull/147

0.18.0 (2023-04-03)
===

* Update to mmtk-core 0.18.0.

0.17.0 (2023-02-17)
===

* Upgrade Rust toolchain to 1.66.1 and MSRV to 1.61.0.
* Use `AtomicUsize` for the mutator counter.
* Update to mmtk-core 0.17.0.

0.16.0 (2022-12-06)
===

* Support MMTk's native mark sweep plan.
* Use MMTk's large code and non moving semantics.
* Update to mmtk-core 0.16.0.

0.15.0 (2022-09-20)
===

* Update to mmtk-core 0.15.0.

0.14.0 (2022-08-08)
===

* Set proper names for MMTk worker threads.
* Inlucde `Cargo.lock` in the repository.
* Update to mmtk-core 0.14.0.

0.13.0 (2022-06-27)
===

* Updates to mmtk-core 0.13.0.

0.12.0 (2022-05-13)
===

* Adds weak reference support (It is disabled by default. Set MMTk option `no_reference_types` to `false` to enable it).
* Updates to mmtk-core 0.12.0.

0.11.0 (2022-04-01)
===

* The JikesRVM submodule is removed from the repo. We now record the VM version
  in `[package.metadata.jikesrvm]` in the Cargo manifest `Cargo.toml`.
* Sets `ObjectModel::OBJECT_REF_OFFSET_BEYOND_CELL` so MMTk can guarantee metadata is set properly
  for object references.
* Updates to mmtk-core 0.11.0.

0.10.0 (2022-02-14)
===

* Updates to mmtk-core 0.10.0.

0.9.0 (2021-12-16)
===

* Updates to mmtk-core 0.9.0.

0.8.0 (2021-11-01)
===

* Refactors current uses of the `llvm_asm!` macro to the new `asm!` macro.
* Updates to mmtk-core 0.8.0.

0.7.0 (2021-09-22)
===

* Updates to mmtk-core 0.7.0.

0.6.0 (2021-08-10)
===

* Added the layout for ImmixAllocator for MutatorContext.
* Updates to mmtk-core 0.6.0

0.5.0 (2021-06-28)
===

* Updates to the JikesRVM upstream commit `0b6002e7d746a829d56c90acfc4bb5c560faf634`.
* Updates `ObjectModel` to support the new metadata structure, where the bindings decide whether to put each per-object metadata on side or in object header.
* Updates to mmtk-core 0.5.0.

0.4.0 (2021-05-17)
===

* Fixes a bug where benchmarks got stock randomly due to a synchronisation issue
* Fixes a bug where edges were pushed more than once (e.g. duplicate edges)
* Adds style checks
* Cleans up some unused code
* Refactors in accordance with the latest changes in `mmtk-core` API
* Updates to mmtk-core 0.4.0


0.3.0 (2021-04-01)
===

* Supports MarkSweep
* Supports finalization
* Updates to mmtk-core 0.3.0


0.2.0 (2020-12-18)
===

* Fixes a bug that causes incorrect return values for syscalls that return a `bool`.
* Updates to mmtk-core 0.2.0.


0.1.0 (2020-11-04)
===

* Supports the following plans from mmtk-core:
  * NoGC
  * SemiSpace

