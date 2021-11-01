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

