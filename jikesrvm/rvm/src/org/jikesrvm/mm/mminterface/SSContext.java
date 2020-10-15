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
import org.mmtk.plan.semispace.SSMutator;
import org.vmmagic.pragma.*;
import org.vmmagic.unboxed.*;
import org.jikesrvm.runtime.Magic;
import org.mmtk.plan.Plan;
import org.mmtk.plan.semispace.SS;
import org.mmtk.utility.alloc.Allocator;

import static org.jikesrvm.runtime.EntrypointHelper.getField;
import static org.jikesrvm.runtime.SysCall.sysCall;
import static org.jikesrvm.runtime.UnboxedSizeConstants.BYTES_IN_WORD;

@Uninterruptible
public class SSContext extends MMTkMutatorContext {
    @Inline
    protected final int getAllocatorTag(int allocator) {
        if (allocator == MMTkAllocator.LOS) {
            return MMTkMutatorContext.TAG_LARGE_OBJECT;
        } else {
            return MMTkMutatorContext.TAG_BUMP_POINTER;
        }
        
    }

    @Inline
    protected final int getAllocatorIndex(int allocator) {
        if (allocator == MMTkAllocator.DEFAULT) {
            return 0;
        } else if (allocator == MMTkAllocator.IMMORTAL || allocator == MMTkAllocator.CODE || allocator == MMTkAllocator.READONLY) {
            return 1;
        } else {
            return 0;
        }
    }
}
