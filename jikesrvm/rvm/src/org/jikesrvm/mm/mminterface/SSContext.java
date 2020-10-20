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
import org.jikesrvm.objectmodel.JavaHeader;

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

    static final byte GC_MARK_BIT_MASK = 1;

    @Override
    @Inline
    public final void postAlloc(ObjectReference ref, ObjectReference typeRef,
                          int bytes, int allocator) {
        allocator = mapAllocator(bytes, allocator);

        int ty = getAllocatorTag(allocator);
        int index = getAllocatorIndex(allocator);        

        if (ty == TAG_LARGE_OBJECT) {
            Address handle = Magic.objectAsAddress(this).plus(MUTATOR_BASE_OFFSET);
            sysCall.sysPostAlloc(handle, ref, typeRef, bytes, allocator);
        } else if (ty == TAG_BUMP_POINTER && index == 1) {
            // Address allocatorBase = Magic.objectAsAddress(this).plus(BUMP_ALLOCATOR_OFFSET).plus(index * BUMP_ALLOCATOR_SIZE);
            // Address space = allocatorBase.plus(BUMP_ALLOCATOR_SPACE).loadAddress();

            // byte oldValue = JavaHeader.readAvailableByte(ref.toObject());
            // byte newValue = (byte) ((oldValue & GC_MARK_BIT_MASK) | space.loadByte()); // space.loadbyte() gets back mark_state
            // JavaHeader.writeAvailableByte(ref.toObject(), newValue);
            Address handle = Magic.objectAsAddress(this).plus(MUTATOR_BASE_OFFSET);
            sysCall.sysPostAlloc(handle, ref, typeRef, bytes, allocator);
        }
    }
}
