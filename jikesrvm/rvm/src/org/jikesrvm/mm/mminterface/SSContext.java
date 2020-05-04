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
public class SSContext extends SSMutator {
    // BumpAllocator (ss)
    @Entrypoint
    Address threadId;
    @Entrypoint
    Address cursor;
    @Entrypoint
    Address limit;
    @Entrypoint
    Address space;
    @Entrypoint
    Address planSS;
    // plan ref
    @Entrypoint
    Address planRef;
    // CommonMutatorContext
    // BumpAllocator (vs)
    @Entrypoint
    Address threadIdImmortal;
    @Entrypoint
    Address cursorImmortal;
    @Entrypoint
    Address limitImmortal;
    @Entrypoint
    Address spaceImmortal;
    @Entrypoint
    Address planImmortal;
    // LargeObjectAllocator
    @Entrypoint
    Address threadIdLos;
    @Entrypoint
    Address spaceLos;
    @Entrypoint
    Address planLos;

    static final Offset threadIdOffset = getField(SSContext.class, "threadId", Address.class).getOffset();
    static final Offset cursorOffset = getField(SSContext.class, "cursor", Address.class).getOffset();
    static final Offset limitOffset = getField(SSContext.class, "limit", Address.class).getOffset();
    static final Offset spaceOffset = getField(SSContext.class, "space", Address.class).getOffset();
    static final Offset planSSOffset = getField(SSContext.class, "planSS", Address.class).getOffset();

    static final Offset planRefOffset = getField(SSContext.class, "planRef", Address.class).getOffset();

    static final Offset threadIdImmortalOffset = getField(SSContext.class, "threadIdImmortal", Address.class).getOffset();
    static final Offset cursorImmortalOffset = getField(SSContext.class, "cursorImmortal", Address.class).getOffset();
    static final Offset limitImmortalOffset = getField(SSContext.class, "limitImmortal", Address.class).getOffset();
    static final Offset spaceImmortalOffset = getField(SSContext.class, "spaceImmortal", Address.class).getOffset();
    static final Offset planImmortalOffset = getField(SSContext.class, "planImmortal", Address.class).getOffset();

    static final Offset threadIdLosOffset = getField(SSContext.class, "threadIdLos", Address.class).getOffset();
    static final Offset spaceLosOffset = getField(SSContext.class, "spaceLos", Address.class).getOffset();
    static final Offset planLosOffset = getField(SSContext.class, "planLos", Address.class).getOffset();

    public Address setBlock(Address mmtkHandle) {
        threadId = mmtkHandle.loadAddress();
        cursor   = mmtkHandle.plus(BYTES_IN_WORD).loadAddress();
        limit    = mmtkHandle.plus(BYTES_IN_WORD * 2).loadAddress();
        space    = mmtkHandle.plus(BYTES_IN_WORD * 3).loadAddress();
        planSS   = mmtkHandle.plus(BYTES_IN_WORD * 4).loadAddress();

        planRef = mmtkHandle.plus(BYTES_IN_WORD * 5).loadAddress();

        threadIdImmortal = mmtkHandle.plus(BYTES_IN_WORD * 6).loadAddress();
        cursorImmortal   = mmtkHandle.plus(BYTES_IN_WORD * 7).loadAddress();
        limitImmortal    = mmtkHandle.plus(BYTES_IN_WORD * 8).loadAddress();
        spaceImmortal    = mmtkHandle.plus(BYTES_IN_WORD * 9).loadAddress();
        planImmortal     = mmtkHandle.plus(BYTES_IN_WORD * 10).loadAddress();

        threadIdLos = mmtkHandle.plus(BYTES_IN_WORD * 11).loadAddress();
        spaceLos = mmtkHandle.plus(BYTES_IN_WORD * 12).loadAddress();
        planLos = mmtkHandle.plus(BYTES_IN_WORD * 13).loadAddress();
        return Magic.objectAsAddress(this).plus(threadIdOffset);
    }

    @Inline
    public int mapAllocator(int bytes, int origAllocator) {
        if (origAllocator == Plan.ALLOC_DEFAULT)
            return MMTkAllocator.DEFAULT;
        else if (origAllocator == Plan.ALLOC_IMMORTAL)
            return MMTkAllocator.IMMORTAL;
        else if (origAllocator == Plan.ALLOC_LOS)
            return MMTkAllocator.LOS;
        else if (origAllocator == Plan.ALLOC_CODE || origAllocator == Plan.ALLOC_LARGE_CODE)
            // FIXME: Should use CODE. Now I am just testing with IMMORTAL to make things easier.
            // return MMTkAllocator.CODE;
            return MMTkAllocator.IMMORTAL;
        else {
            return MMTkAllocator.IMMORTAL;
        }
    }

    @Override
    public Address alloc(int bytes, int align, int offset, int allocator, int site) {
        allocator = mapAllocator(bytes, allocator);

        if (allocator == SS.ALLOC_SS) {
            Address cursor = this.cursor;
            Address sentinel = this.limit;

            // Align allocation
            Word mask = Word.fromIntSignExtend(align - 1);
            Word negOff = Word.fromIntSignExtend(-offset);

            Offset delta = negOff.minus(cursor.toWord()).and(mask).toOffset();

            Address result = cursor.plus(delta);

            Address newCursor = result.plus(bytes);

            if (newCursor.GT(sentinel)) {
                Address handle = Magic.objectAsAddress(this).plus(threadIdOffset);
                return sysCall.sysAllocSlowBumpMonotoneCopy(handle, bytes, align, offset, allocator);
            } else {
                this.cursor = newCursor;
                return result;
            }
        } else {
            Address handle = Magic.objectAsAddress(this).plus(threadIdOffset);
            return sysCall.sysAlloc(handle, bytes, align, offset, allocator);
        }
    }

    @Override
    public void postAlloc(ObjectReference ref, ObjectReference typeRef,
                          int bytes, int allocator) {
        allocator = mapAllocator(bytes, allocator);

        Address handle = Magic.objectAsAddress(this).plus(threadIdOffset);
        sysCall.sysPostAlloc(handle, ref, typeRef, bytes, allocator);
    }
}
