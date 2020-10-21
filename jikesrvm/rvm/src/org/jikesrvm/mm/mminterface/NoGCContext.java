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

import org.vmmagic.pragma.*;
import org.vmmagic.unboxed.*;

@Uninterruptible
public class NoGCContext extends MMTkMutatorContext {
    // Everything is allocated into BumpAllocator #0 (ImmortalSpace)

    @Inline
    protected final int getAllocatorTag(int allocator) {
        return MMTkMutatorContext.TAG_BUMP_POINTER;
    }
    @Inline
    protected final int getAllocatorIndex(int allocator) {
        return 0;
    }
    @Inline
    protected final int getSpaceTag(int allocator) {
        return IMMORTAL_SPACE;
    }

    @Override
    @Inline
    public final void postAlloc(ObjectReference ref, ObjectReference typeRef,
                          int bytes, int allocator) {
        // We dont need to do anything for post alloc for NoGC
    }
}
