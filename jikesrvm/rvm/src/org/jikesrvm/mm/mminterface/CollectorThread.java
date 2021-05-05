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
import org.jikesrvm.mm.mmtk.ScanThread;
import org.jikesrvm.runtime.Magic;
import org.jikesrvm.scheduler.SystemThread;
import org.mmtk.plan.CollectorContext;
import org.vmmagic.pragma.*;
import org.vmmagic.unboxed.Address;

import static org.jikesrvm.runtime.SysCall.sysCall;

/**
 * System thread used to perform garbage collection work.
 */
@NonMoving
public final class CollectorThread extends SystemThread {

  /***********************************************************************
   *
   * Class variables
   */

  /***********************************************************************
   *
   * Instance variables
   */

  @Entrypoint
  private Address workerInstance = Address.zero();

  public void setWorker(Address worker) {
    rvmThread.assertIsCollector();
    workerInstance = worker;
  }

  /** used by collector threads to hold state during stack scanning */
  private final ScanThread threadScanner = new ScanThread();
  private final RustScanThread rustThreadScanner = new RustScanThread();

  /** @return the thread scanner instance associated with this instance */
  @Uninterruptible
  public ScanThread getThreadScanner() {
    return threadScanner;
  }

  @Uninterruptible
  public RustScanThread getRustThreadScanner() {
    return rustThreadScanner;
  }


  /***********************************************************************
   *
   * Initialization
   */

  /**
   * @param stack The stack this thread will run on
   * @param context the context that will provide the thread's
   *  functionality
   */
  public CollectorThread(byte[] stack, CollectorContext context) {
    super(stack, (VM.BuildWithRustMMTk ? "CollectorThread" : context.getClass().getName()) + " [" + nextId + "]");
    if (!VM.BuildWithRustMMTk) {
      rvmThread.collectorContext = context;
      rvmThread.collectorContext.initCollector(nextId);
    }
    nextId++;
  }

  /** Next collector thread id. Collector threads are not created concurrently. */
  private static int nextId = 0;

  /**
   * Collection entry point. Delegates the real work to MMTk.
   */
  @Override
  @NoOptCompile
  // refs stored in registers by opt compiler will not be relocated by GC
  @BaselineNoRegisters
  // refs stored in registers by baseline compiler will not be relocated by GC, so use stack only
  @BaselineSaveLSRegisters
  // and store all registers from previous method in prologue, so that we can stack access them while scanning this thread.
  @Unpreemptible
  public void run() {
    if (VM.BuildWithRustMMTk) {
      if (workerInstance.EQ(Address.zero())) {
        sysCall.sysStartControlCollector(Magic.objectAsAddress(rvmThread));
      } else {
        sysCall.sysStartWorker(Magic.objectAsAddress(rvmThread), workerInstance);
      }
    } else {
      rvmThread.collectorContext.run();
    }
  }
}

