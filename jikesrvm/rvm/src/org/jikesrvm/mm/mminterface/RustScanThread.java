/*
 *  This file is part of the Jikes RVM project (http://jikesrvm.org).
 *
 *  This file is licensed to You under the Eclipse Public License (EPL);
 *  You may not use this file except in compliance with the License. You
 *  may obtain a copy of the License at
 *
 *      http://www.opensource.org/licenses/eclipse-1.0.php
 *
 *  See the COPYRIGHT.txt file distributed with this work for information
 *  regarding copyright ownership.
 */
package org.jikesrvm.mm.mminterface;

import org.jikesrvm.VM;
import org.jikesrvm.architecture.AbstractRegisters;
import org.jikesrvm.architecture.StackFrameLayout;
import org.jikesrvm.compilers.common.CompiledMethod;
import org.jikesrvm.compilers.common.CompiledMethods;
import org.jikesrvm.mm.mmtk.ObjectModel;
import org.jikesrvm.runtime.Magic;
import org.jikesrvm.runtime.RuntimeEntrypoints;
import org.jikesrvm.scheduler.RVMThread;
import org.vmmagic.pragma.Entrypoint;
import org.vmmagic.pragma.Inline;
import org.vmmagic.pragma.Uninterruptible;
import org.vmmagic.pragma.Untraced;
import org.vmmagic.unboxed.Address;
import org.vmmagic.unboxed.Word;
import org.vmmagic.unboxed.ObjectReference;
import org.vmmagic.unboxed.Offset;
import static org.jikesrvm.runtime.SysCall.sysCall;
import static org.jikesrvm.runtime.UnboxedSizeConstants.LOG_BYTES_IN_WORD;

/**
 * Class that supports scanning thread stacks for references during
 * collections. References are located using GCMapIterators and are
 * inserted into a set of root locations.  Optionally, a set of
 * interior pointer locations associated with the object is created.<p>
 *
 * Threads, stacks, jni environments, and register objects have a
 * complex interaction in terms of scanning.  The operation of
 * scanning the stack reveals not only roots inside the stack but also
 * the state of the register objects's gprs and the JNI refs array.
 * They are all associated via the thread object, making it natural
 * for scanThread to be considered a single operation with the method
 * directly accessing these objects via the thread object's
 * fields. <p>
 *
 * One pitfall occurs when scanning the thread object (plus
 * dependents) when not all of the objects have been copied.  Then it
 * may be that the innards of the register object has not been copied
 * while the stack object has.  The result is that an inconsistent set
 * of slots is reported.  In this case, the copied register object may
 * not be correct if the copy occurs after the root locations are
 * discovered but before those locations are processed. In essence,
 * all of these objects form one logical unit but are physically
 * separated so that sometimes only part of it has been copied causing
 * the scan to be incorrect. <p>
 *
 * The caller of the stack scanning routine must ensure that all of
 * these components's descendants are consistent (all copied) when
 * this method is called. <p>
 *
 * <i>Code locations:</i> Identifying pointers <i>into</i> code
 * objects is essential if code objects are allowed to move (and if
 * the code objects were not otherwise kept alive, it would be
 * necessary to ensure the liveness of the code objects). A code
 * pointer is the only case in which we have interior pointers
 * (pointers into the inside of objects).  For such pointers, two
 * things must occur: first the pointed to object must be kept alive,
 * and second, if the pointed to object is moved by a copying
 * collector, the pointer into the object must be adjusted so it now
 * points into the newly copied object
 */
@Uninterruptible public final class RustScanThread {

  /***********************************************************************
   *
   * Class variables
   */

  /** quietly validates each ref reported by map iterators */
  // FIXME: Enable this once rust-mmtk mmaps in the correct address range
  static final boolean VALIDATE_REFS = VM.VerifyAssertions && !VM.BuildWithRustMMTk;

  /*
   * debugging options to produce printout during scanStack
   * MULTIPLE GC THREADS WILL PRODUCE SCRAMBLED OUTPUT so only
   * use these when running with PROCESSORS=1
   */
  private static final int DEFAULT_VERBOSITY = 0 /*0*/;
  private static final int FAILURE_VERBOSITY = 4;

  // FIXME: GC options
  private static final boolean USE_SHORT_STACK_SCANS = false;
  private static final boolean USE_RETURN_BARRIER = false;

  /***********************************************************************
   *
   * Instance variables
   */

  /**
   *
   */
  private final GCMapIteratorGroup iteratorGroup = new GCMapIteratorGroup();
  @Untraced
  private GCMapIterator iterator;
  @Untraced
  private boolean processCodeLocations;
  private Address report_edges;
  private Address report_edges_extra_data;
  @Untraced
  private RVMThread thread;
  private Address ip, fp, prevFp, initialIPLoc, topFrame;
  @Untraced
  private CompiledMethod compiledMethod;
  private int compiledMethodType;
  private boolean failed;
  private boolean reinstallReturnBarrier;

  private Address edges;
  private Word size = Word.zero();
  // The buffer size agreed between the Rust part and the Java part of the binding.
  // See the constant EDGES_BUFFER_CAPACITY in scanning.rs.
  public static final Word EDGES_BUFFER_CAPACITY = Word.fromIntZeroExtend(4096);

  /***********************************************************************
   *
   * Thread scanning
   */

  /**
   * Scan a thread, placing the addresses of pointers into supplied buffers.
   *
   * @param thread The thread to be scanned
   * @param report_edges The native call-back function to use for reporting locations.
   * @param report_edges_extra_data Extra data passed to the report_edges call-back.
   * @param processCodeLocations Should code locations be processed?
   * @param newRootsSufficient Is a partial stack scan sufficient, or must we do a full scan?
   */
  @Entrypoint
  public static void scanThread(RVMThread thread,
                                Address report_edges,
                                Address report_edges_extra_data,
                                boolean processCodeLocations,
                                boolean newRootsSufficient) {
    if (DEFAULT_VERBOSITY >= 1) {
      VM.sysWriteln("scanning ",thread.getThreadSlot());
    }

    /* get the gprs associated with this thread */
    AbstractRegisters regs = thread.getContextRegisters();
    Address gprs = Magic.objectAsAddress(regs.getGPRs());

    Address ip = regs.getInnermostInstructionAddress();
    Address fp = regs.getInnermostFramePointer();
    regs.clear();
    regs.setInnermost(ip,fp);
    scanThread(thread, report_edges, report_edges_extra_data, processCodeLocations, gprs, Address.zero(), newRootsSufficient);
  }

  /**
   * A more general interface to thread scanning, which permits the
   * scanning of stack segments which are dislocated from the thread
   * structure.
   *
   * @param thread The thread to be scanned
   * @param report_edges The native call-back function to use for reporting locations.
   * @param report_edges_extra_data Extra data passed to the report_edges call-back.
   * @param processCodeLocations Should code locations be processed?
   * @param gprs The general purpose registers associated with the
   * stack being scanned (normally extracted from the thread).
   * @param topFrame The top frame of the stack being scanned, or zero
   * if this is to be inferred from the thread (normally the case).
   * @param newRootsSufficent Is a partial stack scan sufficient, or must we do a full scan?
   */
  private static void scanThread(RVMThread thread,
                                 Address report_edges,
                                 Address report_edges_extra_data,
                                 boolean processCodeLocations,
                                 Address gprs,
                                 Address topFrame,
                                 boolean newRootsSufficent) {
    // figure out if the thread should be scanned at all; if not, exit
    if (thread.getExecStatus() == RVMThread.NEW || thread.getIsAboutToTerminate()) {
      return;
    }
    /* establish ip and fp for the stack to be scanned */
    Address ip, fp, initialIPLoc;
    if (topFrame.isZero()) { /* implicit top of stack, inferred from thread */
      ip = thread.getContextRegisters().getInnermostInstructionAddress();
      fp = thread.getContextRegisters().getInnermostFramePointer();
      initialIPLoc = thread.getContextRegisters().getIPLocation();
    } else {                 /* top frame explicitly defined */
      ip = Magic.getReturnAddress(topFrame, thread);
      fp = Magic.getCallerFramePointer(topFrame);
      initialIPLoc = thread.getContextRegisters().getIPLocation(); // FIXME
    }

    /* Grab the ScanThread instance associated with this thread */
    RustScanThread scanner = RVMThread.getCurrentThread().getCollectorThread().getRustThreadScanner();

    /* Explicitly establish the stopping point for this scan (not necessarily the bottom of stack) */
    Address sentinelFp = newRootsSufficent && USE_SHORT_STACK_SCANS ? thread.getNextUnencounteredFrame() : StackFrameLayout.getStackFrameSentinelFP();

    /* stack trampoline will be freshly reinstalled at end of thread scan */
    if (USE_RETURN_BARRIER || USE_SHORT_STACK_SCANS) {
      thread.deInstallStackTrampoline();
    }

    /* scan the stack */
    scanner.startScan(report_edges, report_edges_extra_data, processCodeLocations, thread, gprs, ip, fp, initialIPLoc, topFrame, sentinelFp);
  }

  @Inline
  private void reportEdge(Address edge) {
    // Push value
    Word cursor = this.size;
    this.size = cursor.plus(Word.one());
    if (VM.VerifyAssertions) VM._assert(!this.edges.isZero());
    this.edges.plus(cursor.toInt() << LOG_BYTES_IN_WORD).store(edge);
    // Flush if full
    if (cursor.GE(EDGES_BUFFER_CAPACITY)) {
      flush();
    }
  }

  private void flush() {
    if (!edges.isZero() && !size.isZero()) {
      edges = sysCall.sysDynamicCall3(report_edges, edges.toWord(), size, report_edges_extra_data.toWord());
      size = Word.zero();
    }
  }

  /**
   * Initializes a ScanThread instance, and then scans a stack
   * associated with a thread, and places references in deques (one for
   * object pointers, one for interior code pointers).  If the thread
   * scan fails, the thread is rescanned verbosely and a failure
   * occurs.<p>
   *
   * The various state associated with stack scanning is captured by
   * instance variables of this type, which are initialized here.
   *
   * @param report_edges The native call-back function to use for reporting locations.
   * @param report_edges_extra_data Extra data passed to the report_edges call-back.
   * @param processCodeLocations whether to process parts of the thread
   *  that could point to code (e.g. exception registers).
   * @param thread Thread for the thread whose stack is being scanned
   * @param gprs The general purpose registers associated with the
   * stack being scanned (normally extracted from the thread).
   * @param ip The instruction pointer for the top frame of the stack
   * we're about to scan.
   * @param fp The frame pointer for the top frame of the stack we're
   * about to scan.
   * @param initialIPLoc the address of the initial location of the instruction
   *  pointer
   * @param topFrame The top frame of the stack being scanned, or zero
   * if this is to be inferred from the thread (normally the case).
   * @param sentinelFp The frame pointer at which the stack scan should stop.
   */
  private void startScan(Address report_edges,
                         Address report_edges_extra_data,
                         boolean processCodeLocations,
                         RVMThread thread, Address gprs, Address ip,
                         Address fp, Address initialIPLoc, Address topFrame,
                         Address sentinelFp) {
    this.report_edges = report_edges;
    this.report_edges_extra_data = report_edges_extra_data;
    this.size = Word.zero();
    this.edges = sysCall.sysDynamicCall3(report_edges, Word.zero(), Word.zero(), report_edges_extra_data.toWord());

    this.processCodeLocations = processCodeLocations;
    this.thread = thread;
    this.failed = false;
    this.ip = ip;
    this.fp = fp;
    this.initialIPLoc = initialIPLoc;
    this.topFrame = topFrame;
    scanThreadInternal(gprs, DEFAULT_VERBOSITY, sentinelFp);
    if (failed) {
       /* reinitialize and rescan verbosely on failure */
      this.ip = ip;
      this.fp = fp;
      this.topFrame = topFrame;
      scanThreadInternal(gprs, FAILURE_VERBOSITY, sentinelFp);
      VM.sysFail("Error encountered while scanning stack");
    }
    flush();
    if (!edges.isZero()) {
      sysCall.release_buffer(edges);
    }
  }

  /**
   * The main stack scanning loop.<p>
   *
   * Walk the stack one frame at a time, top (lo) to bottom (hi),<p>
   *
   * @param gprs The general purpose registers associated with the
   * stack being scanned (normally extracted from the thread).
   * @param verbosity The level of verbosity to be used when
   * performing the scan.
   * @param sentinelFp the frame pointer at which the stack scan should stop
   */
  private void scanThreadInternal(Address gprs, int verbosity, Address sentinelFp) {
    if (false) {
      VM.sysWriteln("Scanning thread ",thread.getThreadSlot()," from thread ",RVMThread.getCurrentThreadSlot());
    }
    /*if (verbosity >= 2) {
      Log.writeln("--- Start Of Stack Scan ---\n");
      Log.write("Thread #");
      Log.writeln(thread.getThreadSlot());
    }*/
    if (VM.VerifyAssertions) assertImmovableInCurrentCollection();

    /* first find any references to exception handlers in the registers */
    getHWExceptionRegisters();

    /* reinitialize the stack iterator group */
    iteratorGroup.newStackWalk(thread, gprs);

    if (verbosity >= 2) dumpTopFrameInfo(verbosity);

    /* scan each frame if a non-empty stack */
    if (fp.NE(StackFrameLayout.getStackFrameSentinelFP())) {
      prevFp = Address.zero();
      reinstallReturnBarrier = USE_RETURN_BARRIER || USE_SHORT_STACK_SCANS;
      /* At start of loop:
         fp -> frame for method invocation being processed
         ip -> instruction pointer in the method (normally a call site) */
      while (Magic.getCallerFramePointer(fp).NE(sentinelFp)) {
        if (false) {
          VM.sysWriteln("Thread ",RVMThread.getCurrentThreadSlot()," at fp = ",fp);
        }
        prevFp = scanFrame(verbosity);
        ip = Magic.getReturnAddress(fp, thread);
        fp = Magic.getCallerFramePointer(fp);
      }
    }

    /* If a thread started via createVM or attachVM, base may need scaning */
    // TODO implement this if necessary. It was previously only implemented for
    // AIX which is no longer supported.
  }

  /**
   * When an exception occurs, registers are saved temporarily.  If
   * the stack being scanned is in this state, we need to scan those
   * registers for code pointers.  If the codeLocations deque is null,
   * then scanning for code pointers is not required, so we don't need
   * to do anything. (SB: Why only code pointers?).
   * <p>
   * Dave G:  The contents of the GPRs of the exceptionRegisters
   * are handled during normal stack scanning
   * (@see org.jikesrvm.runtime.compilers.common.HardwareTrapCompiledMethod.
   * It looks to me like the main goal of this method is to ensure that the
   * method in which the trap happened isn't treated as dead code and collected
   * (if it's been marked as obsolete, we are setting its activeOnStackFlag below).
   */
  private void getHWExceptionRegisters() {
    AbstractRegisters exReg = thread.getExceptionRegisters();
    if (processCodeLocations && exReg.getInUse()) {
      Address ip = exReg.getIP();
      CompiledMethod compiledMethod = CompiledMethods.findMethodForInstruction(ip);
      if (VM.VerifyAssertions) {
        VM._assert(compiledMethod != null);
        VM._assert(compiledMethod.containsReturnAddress(ip));
      }
      compiledMethod.setActiveOnStack();
      ObjectReference code = ObjectReference.fromObject(compiledMethod.getEntryCodeArray());
      Address ipLoc = exReg.getIPLocation();
      if (VM.VerifyAssertions) VM._assert(ip.EQ(ipLoc.loadAddress()));
      processCodeLocation(code, ipLoc);
    }
  }

  /**
   * Push a code pointer location onto the code locations deque,
   * optionally performing a sanity check first.<p>
   *
   * @param code The code object into which this interior pointer points
   * @param ipLoc The location of the pointer into this code object
   */
  @Inline
  private void processCodeLocation(ObjectReference code, Address ipLoc) {
    if (VALIDATE_REFS) {
      Address ip = ipLoc.loadAddress();
      Offset offset = ip.diff(code.toAddress());

      if (offset.sLT(Offset.zero()) ||
          offset.sGT(Offset.fromIntZeroExtend(ObjectModel.getObjectSize(code)))) {
        if (!failed) failed = true;
      }
    }
    reportEdge(ipLoc);
    // sysCall.sysProcessInteriorEdge(trace, code, ipLoc, true);
  }

  /***********************************************************************
   *
   * Frame scanning methods
   */

  /**
   * Scan the current stack frame.<p>
   *
   * First the various iterators are set up, then the frame is scanned
   * for regular pointers, before scanning for code pointers.  The
   * iterators are then cleaned up, and native frames skipped if
   * necessary.
   *
   * @param verbosity The level of verbosity to be used when
   * performing the scan.
   * @return the frame pointer of the frame that was just scanned
   */
  private Address scanFrame(int verbosity) {
    /* set up iterators etc, and skip the frame if appropriate */
    if (!setUpFrame(verbosity)) return fp;

    /* scan the frame for object pointers */
    scanFrameForObjects(verbosity);

    /* scan the frame for pointers to code */
    if (processCodeLocations && compiledMethodType != CompiledMethod.TRAP)
      processFrameForCode(verbosity);

    iterator.cleanupPointers();

    if (compiledMethodType != CompiledMethod.TRAP) {
      /* skip preceding native frames if this frame is a native bridge */
      if (compiledMethod.getMethod().getDeclaringClass().hasBridgeFromNativeAnnotation()) {
        fp = RuntimeEntrypoints.unwindNativeStackFrameForGC(fp);
      }

      /* reinstall the return barrier if necessary (and verbosity indicates that this is a regular scan) */
      if (reinstallReturnBarrier && verbosity == DEFAULT_VERBOSITY) {
        thread.installStackTrampolineBridge(fp);
        reinstallReturnBarrier = false;
      }
    }
    return fp;
  }

  /**
   * Set up to scan the current stack frame.  This means examining the
   * frame to discover the method being invoked and then retrieving
   * the associated metadata (stack maps etc).  Certain frames should
   * not be scanned---these are identified and skipped.
   *
   * @param verbosity The level of verbosity to be used when
   * performing the scan.
   * @return True if the frame should be scanned, false if it should
   * be skipped.
   */
  private boolean setUpFrame(int verbosity) {
    /* get the compiled method ID for this frame */
    int compiledMethodId = Magic.getCompiledMethodID(fp);

    /* skip "invisible" transition frames generated by reflection and JNI) */
    if (compiledMethodId == StackFrameLayout.getInvisibleMethodID()) {
      return false;
    }

    /* establish the compiled method */
    compiledMethod = CompiledMethods.getCompiledMethod(compiledMethodId);
    compiledMethod.setActiveOnStack();  // prevents code from being collected

    compiledMethodType = compiledMethod.getCompilerType();

    /* get the code associated with this frame */
    if (RVMThread.DEBUG_STACK_TRAMPOLINE) VM.sysWriteln(thread.getId(), fp, compiledMethod.getMethod());
    Offset offset = compiledMethod.getInstructionOffset(ip);

    /* initialize MapIterator for this frame */
    iterator = iteratorGroup.selectIterator(compiledMethod);
    iterator.setupIterator(compiledMethod, offset, fp);

    return true;
  }

  /**
   * Identify all the object pointers stored as local variables
   * associated with (though not necessarily strictly within!) the
   * current frame.  Loop through the GC map iterator, getting the
   * address of each object pointer, adding them to the root locations
   * deque.<p>
   *
   * NOTE: Because of the callee save policy of the optimizing
   * compiler, references associated with a given frame may be in
   * callee stack frames (lower memory), <i>outside</i> the current
   * frame.  So the iterator may return locations that are outside the
   * frame being scanned.
   *
   * @param verbosity The level of verbosity to be used when
   * performing the scan.
   */
  private void scanFrameForObjects(int verbosity) {
    for (Address refaddr = iterator.getNextReferenceAddress();
         !refaddr.isZero();
         refaddr = iterator.getNextReferenceAddress()) {
      if (VALIDATE_REFS) checkReference(refaddr, verbosity);
      if (verbosity >= 4) dumpRef(refaddr, verbosity);
      reportEdge(refaddr);
      // reportDelayedRootEdge(trace, refaddr);
    }
  }

  /**
   * Identify all pointers into code pointers associated with a frame.
   * There are two cases to be considered: a) the instruction pointer
   * associated with each frame (stored in the thread's metadata for
   * the top frame and as a return address for all subsequent frames),
   * and b) local variables on the stack which happen to be pointers
   * to code.<p>
   *
   * FIXME: SB: Why is it that JNI frames are skipped when considering
   * top of stack frames, while boot image frames are skipped when
   * considering other frames.  Shouldn't they both be considered in
   * both cases?
   *
   * @param verbosity The level of verbosity to be used when
   * performing the scan.
   */
  private void processFrameForCode(int verbosity) {
    /* get the code object associated with this frame */
    ObjectReference code = ObjectReference.fromObject(compiledMethod.getEntryCodeArray());

    pushFrameIP(code, verbosity);
    scanFrameForCode(code);
  }

  /**
   * Push the instruction pointer associated with this frame onto the
   * code locations deque.<p>
   *
   * A stack frame represents an execution context, and thus has an
   * instruction pointer associated with it.  In the case of the top
   * frame, the instruction pointer is captured by the IP register,
   * which is preserved in the thread data structure at thread switch
   * time.  In the case of all non-top frames, the next instruction
   * pointer is stored as the return address for the <i>previous</i>
   * frame.<p>
   *
   * The address of the code pointer is pushed onto the code locations
   * deque along with the address of the code object into which it
   * points (both are required since the former is an internal
   * pointer).<p>
   *
   * The code pointers are updated later (after stack scanning) when
   * the code locations deque is processed. The pointer from RVMMethod
   * to the code object is not updated until after stack scanning, so
   * the pointer to the (uncopied) code object is available throughout
   * the stack scanning process, which enables interior pointer
   * offsets to be correctly computed.
   *
   * @param code start address of the machine code array associated
   *  with the method
   * @param verbosity The level of verbosity to be used when
   * performing the scan.
   */
  private void pushFrameIP(ObjectReference code, int verbosity) {
    if (prevFp.isZero()) {  /* top of stack: IP in thread state */
      /* skip native code, as it is not (cannot be) moved */
      if (compiledMethodType != CompiledMethod.JNI)
        processCodeLocation(code, initialIPLoc);
    } else {  /* below top of stack: IP is return address, in prev frame */
      Address returnAddressLoc = Magic.getReturnAddressLocation(prevFp);
      Address returnAddress = returnAddressLoc.loadAddress();
      /* skip boot image code, as it is not (cannot be) moved */
      if (!DebugUtil.addrInBootImage(returnAddress))
        processCodeLocation(code, returnAddressLoc);
    }
  }

  /**
   * Scan this frame for internal code pointers.  The GC map iterator
   * is used to identify any local variables (stored on the stack)
   * which happen to be pointers into code.<p>
   *
   * @param code The code object associated with this frame.
   */
  private void scanFrameForCode(ObjectReference code) {
    iterator.reset();
    for (Address retaddrLoc = iterator.getNextReturnAddressAddress();
         !retaddrLoc.isZero();
         retaddrLoc = iterator.getNextReturnAddressAddress())
      processCodeLocation(code, retaddrLoc);
  }

  /***********************************************************************
   *
   * Debugging etc
   */

  /**
   * Assert that the stack is immovable.<p>
   *
   * Currently we do not allow stacks to be moved within the heap.  If
   * a stack contains native stack frames, then it is impossible for
   * us to safely move it.  Prior to the implementation of JNI, Jikes
   * RVM did allow the GC system to move thread stacks, and called a
   * special fixup routine, thread.fixupMovedStack to adjust all of
   * the special interior pointers (SP, FP).  If we implement split C
   * &amp; Java stacks then we could allow the Java stacks to be moved,
   * but we can't move the native stack.
   */
  private void assertImmovableInCurrentCollection() {
    // This method is guarded by VM.VerifyAssertions. Our Checkstyle assertion
    // plugin does not recognize this because it does not track calls. Therefore,
    // switch off Checkstyle for this method.

    //CHECKSTYLE:OFF
    // VM._assert(sysCall.sysWillNotMoveInCurrentCollection(trace, ObjectReference.fromObject(thread.getStack())));
    // VM._assert(sysCall.sysWillNotMoveInCurrentCollection(trace, ObjectReference.fromObject(thread)));
    // VM._assert(sysCall.sysWillNotMoveInCurrentCollection(trace, ObjectReference.fromObject(thread.getStack())));
    // VM._assert(thread.getJNIEnv() == null || sysCall.sysWillNotMoveInCurrentCollection(trace, ObjectReference.fromObject(thread.getJNIEnv())));
    // VM._assert(thread.getJNIEnv() == null || thread.getJNIEnv().refsArray() == null || sysCall.sysWillNotMoveInCurrentCollection(trace, ObjectReference.fromObject(thread.getJNIEnv().refsArray())));
    // VM._assert(sysCall.sysWillNotMoveInCurrentCollection(trace,ObjectReference.fromObject(thread.getContextRegisters())));
    // VM._assert(sysCall.sysWillNotMoveInCurrentCollection(trace,ObjectReference.fromObject(thread.getContextRegisters().getGPRs())));
    // VM._assert(sysCall.sysWillNotMoveInCurrentCollection(trace,ObjectReference.fromObject(thread.getExceptionRegisters())));
    // VM._assert(sysCall.sysWillNotMoveInCurrentCollection(trace,ObjectReference.fromObject(thread.getExceptionRegisters().getGPRs())));
    //CHECKSTYLE:ON
  }

  /**
   * Print out the basic information associated with the top frame on
   * the stack.
   *
   * @param verbosity The level of verbosity to be used when
   * performing the scan.
   */
  private void dumpTopFrameInfo(int verbosity) {
    if (verbosity >= 3 && thread.getJNIEnv() != null)
      thread.getJNIEnv().dumpJniRefsStack();
  }

  /**
   * Print out information associated with a reference.
   *
   * @param refaddr The address of the reference in question.
   * @param verbosity The level of verbosity to be used when
   * performing the scan.
   */
  private void dumpRef(Address refaddr, int verbosity) {
    ObjectReference ref = refaddr.loadObjectReference();
    VM.sysWrite(refaddr);
    if (verbosity >= 5) {
      VM.sysWrite(":");
      MemoryManager.dumpRef(ref);
    } else
      VM.sysWriteln();
  }

  /**
   * Check that a reference encountered during scanning is valid.  If
   * the reference is invalid, dump stack and die.
   *
   * @param refaddr The address of the reference in question.
   * @param verbosity The level of verbosity to be used when
   * performing the scan.
   */
  private void checkReference(Address refaddr, int verbosity) {
    ObjectReference ref = refaddr.loadObjectReference();
    if (!MemoryManager.validRef(ref)) {
      MemoryManager.dumpRef(ref);
      RVMThread.dumpStack(ip, fp);
      /* dump stack starting at top */
      Address top_ip = thread.getContextRegisters().getInnermostInstructionAddress();
      Address top_fp = thread.getContextRegisters().getInnermostFramePointer();
      RVMThread.dumpStack(top_ip, top_fp);
      Offset offset = compiledMethod.getInstructionOffset(ip);
      iterator = iteratorGroup.selectIterator(compiledMethod);
      iterator.setupIterator(compiledMethod, offset, fp);
      int i = 0;
      for (Address addr = iterator.getNextReferenceAddress();
           !addr.isZero();
           addr = iterator.getNextReferenceAddress()) {
        ObjectReference ref2 = addr.loadObjectReference();
        MemoryManager.dumpRef(ref2);
      }
      VM.sysFail("\n\nScanStack: Detected bad GC map; exiting RVM with fatal error");
    }
  }

  /**
   * Check that a reference encountered during scanning is valid.  If
   * the reference is invalid, dump stack and die.
   *
   * @param refaddr The address of the reference in question.
   */
  private static void checkReference(Address refaddr) {
    ObjectReference ref = refaddr.loadObjectReference();
    if (!MemoryManager.validRef(ref)) {
      MemoryManager.dumpRef(ref);
      RVMThread.dumpStack();
      VM.sysFail("\n\nScanStack: Detected bad GC map; exiting RVM with fatal error");
    }
  }
}
