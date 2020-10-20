package org.jikesrvm.mm.mminterface;

import org.vmmagic.pragma.*;
import org.vmmagic.unboxed.*;
import org.mmtk.plan.MutatorContext;
import org.mmtk.plan.Plan;
import org.jikesrvm.VM;
import org.jikesrvm.runtime.EntrypointHelper;
import org.jikesrvm.runtime.Magic;
import static org.jikesrvm.runtime.SysCall.sysCall;
import static org.jikesrvm.runtime.UnboxedSizeConstants.BYTES_IN_WORD;

@Uninterruptible
public abstract class MMTkMutatorContext extends MutatorContext {
    // The layout of this mutator section should be the same as struct Mutator in MMTk core. 
    // Due to the facdt that RJava does not support unboxed struct or nested struct, we have to layout all the fields here.

    // Mutator section starts

    // Allocators
    // 5 x BumpAllocator (5 x 6 words)

    // Bump Allocator 0
    @Entrypoint
    Address bumpAllocator0Tls;
    @Entrypoint
    Address bumpAllocator0Cursor;
    @Entrypoint
    Address bumpAllocator0Limit;
    @Entrypoint
    Address bumpAllocator0Space;
    @Entrypoint
    Address bumpAllocator0SpaceFat;
    @Entrypoint
    Address bumpAllocator0Plan;

    // Bump Allocator 1
    @Entrypoint
    Address bumpAllocator1Tls;
    @Entrypoint
    Address bumpAllocator1Cursor;
    @Entrypoint
    Address bumpAllocator1Limit;
    @Entrypoint
    Address bumpAllocator1Space;
    @Entrypoint
    Address bumpAllocator1SpaceFat;
    @Entrypoint
    Address bumpAllocator1Plan;

    // Bump Allocator 2
    @Entrypoint
    Address bumpAllocator2Tls;
    @Entrypoint
    Address bumpAllocator2Cursor;
    @Entrypoint
    Address bumpAllocator2Limit;
    @Entrypoint
    Address bumpAllocator2Space;
    @Entrypoint
    Address bumpAllocator2SpaceFat;
    @Entrypoint
    Address bumpAllocator2Plan;

    // Bump Allocator 3
    @Entrypoint
    Address bumpAllocator3Tls;
    @Entrypoint
    Address bumpAllocator3Cursor;
    @Entrypoint
    Address bumpAllocator3Limit;
    @Entrypoint
    Address bumpAllocator3Space;
    @Entrypoint
    Address bumpAllocator3SpaceFat;
    @Entrypoint
    Address bumpAllocator3Plan;

    // Bump Allocator 4
    @Entrypoint
    Address bumpAllocator4Tls;
    @Entrypoint
    Address bumpAllocator4Cursor;
    @Entrypoint
    Address bumpAllocator4Limit;
    @Entrypoint
    Address bumpAllocator4Space;
    @Entrypoint
    Address bumpAllocator4SpaceFat;
    @Entrypoint
    Address bumpAllocator4Plan;

    // word size
    static final int MAX_BUMP_ALLOCATORS = 5;
    static final int BUMP_ALLOCATOR_SIZE = 6 * BYTES_IN_WORD;
    // word offset from base
    static final int BUMP_ALLOCATOR_TLS = 0;
    static final int BUMP_ALLOCATOR_CURSOR = BYTES_IN_WORD;
    static final int BUMP_ALLOCATOR_LIMIT = BYTES_IN_WORD * 2;
    static final int BUMP_ALLOCATOR_SPACE = BYTES_IN_WORD * 3;
    static final int BUMP_ALLOCATOR_SPACE_FAT = BYTES_IN_WORD * 4;
    static final int BUMP_ALLOCATOR_PLAN = BYTES_IN_WORD * 5;
    // 1 x LargeObjectAllocator (1 x 3 words)
    @Entrypoint
    Address largeObjectAllocator0Tls;
    @Entrypoint
    Address largeObjectAllocator0Space;
    @Entrypoint
    Address largeObjectAllocator0Plan;
    // word size
    static final int MAX_LARGE_OBJECT_ALLOCATORS = 1;
    static final int LARGE_OBJECT_ALLOCATOR_SIZE = 3 * BYTES_IN_WORD;

    // mutator_tls
    @Entrypoint
    Address mutatorTls;
    // plan
    @Entrypoint
    Address plan;

    // MutatorConfig
    // allocator_mapping
    @Entrypoint
    Address allocatorMapping;
    // space_mapping
    @Entrypoint
    Address spaceMapping;
    // collection_phase_func (fat pointer)
    @Entrypoint
    Address collection_phase_func;
    Address collection_phase_func_fat;

    @Entrypoint
    Address mutatorEnd;

    // Mutator section ends

    // the size of this mutator section
    static final int MUTATOR_SIZE = MAX_BUMP_ALLOCATORS * BUMP_ALLOCATOR_SIZE + MAX_LARGE_OBJECT_ALLOCATORS * LARGE_OBJECT_ALLOCATOR_SIZE + 6 * BYTES_IN_WORD;    
    // the base offset of this mutator section
    static final Offset MUTATOR_BASE_OFFSET = EntrypointHelper.getField(MMTkMutatorContext.class, "bumpAllocator0Tls", Address.class).getOffset();
    static final Offset BUMP_ALLOCATOR_OFFSET = EntrypointHelper.getField(MMTkMutatorContext.class, "bumpAllocator0Tls", Address.class).getOffset();
    static final Offset LARGE_OBJECT_ALLOCATOR_OFFSET = EntrypointHelper.getField(MMTkMutatorContext.class, "largeObjectAllocator0Tls", Address.class).getOffset();
    static final Offset MUTATOR_END = EntrypointHelper.getField(MMTkMutatorContext.class, "mutatorEnd", Address.class).getOffset();

    // tag to use for allocator selector
    public static final int TAG_BUMP_POINTER = 0;
    public static final int TAG_LARGE_OBJECT = 1;

    // for space selector
    public static final int IMMORTAL_SPACE = 0;
    public static final int COPY_SPACE = 1;
    public static final int LARGE_OBJECT_SPACE = 2;

    // The implementation of these two method should allow the compiler to do partial evaluation.
    @Inline
    protected abstract int getAllocatorTag(int allocator);
    @Inline
    protected abstract int getAllocatorIndex(int allocator);
    @Inline
    protected abstract int getSpaceTag(int allocator);
    @Inline
    public final int mapAllocator(int bytes, int origAllocator) {
        if (origAllocator == Plan.ALLOC_DEFAULT)
            return MMTkAllocator.DEFAULT;
        else if (origAllocator == Plan.ALLOC_IMMORTAL)
            return MMTkAllocator.IMMORTAL;
        else if (origAllocator == Plan.ALLOC_LOS)
            return MMTkAllocator.LOS;
        else if (origAllocator == Plan.ALLOC_CODE || origAllocator == Plan.ALLOC_LARGE_CODE)
            // FIXME: Should use CODE. However, mmtk-core hasn't implemented the CODE allocator.
            // return MMTkAllocator.CODE;
            return MMTkAllocator.CODE;
        else {
            return MMTkAllocator.IMMORTAL;
        }
    }

    @Override
    @Inline
    public final Address alloc(int bytes, int align, int offset, int allocator, int site) {
        allocator = mapAllocator(bytes, allocator);

        int ty = getAllocatorTag(allocator);
        int index = getAllocatorIndex(allocator);

        // VM.sysWrite("-------alloc() with allocator ty", ty); VM.sysWriteln("index", index);
        // sysCall.sysConsoleFlushErrorAndTrace();
        // VM.sysWriteln("assumed size (bytes) = ", MUTATOR_SIZE);
        // VM.sysWriteln("actual size (bytes) = ", MUTATOR_END.minus(MUTATOR_BASE_OFFSET).toInt());
        // VM.sysWriteln("JikesRVM mutator size = ", MUTATOR_SIZE);
        // VM._assert(MUTATOR_SIZE == MUTATOR_END.minus(MUTATOR_BASE_OFFSET).toInt(), "Mutator size does not match");

        // VM._assert(ty == 0);
        // VM._assert(index == 0);
        
        if (ty == TAG_BUMP_POINTER) {
            return bumpAllocatorFastPath(bytes, align, offset, allocator, index);
        } else if (ty == TAG_LARGE_OBJECT) {
            return slowPath(bytes, align, offset, allocator);
        } else {
            VM.sysFail("unimplemented");
            return Address.zero();
        }
    }

    @Inline
    protected final Address bumpAllocatorFastPath(int bytes, int align, int offset, int allocator, int allocatorIndex) {
        // Align allocation
        Word mask = Word.fromIntSignExtend(align - 1);
        Word negOff = Word.fromIntSignExtend(-offset);

        Address allocatorBase = Magic.objectAsAddress(this).plus(BUMP_ALLOCATOR_OFFSET).plus(allocatorIndex * BUMP_ALLOCATOR_SIZE);
        // VM.sysWriteln("this = ", Magic.objectAsAddress(this));
        // VM.sysWriteln("mutator = ", Magic.objectAsAddress(this).plus(BUMP_ALLOCATOR_OFFSET));
        // VM.sysWriteln("allocator = ", Magic.objectAsAddress(this).plus(BUMP_ALLOCATOR_OFFSET).plus(allocatorIndex * BUMP_ALLOCATOR_SIZE));
        
        Address cursor = allocatorBase.plus(BUMP_ALLOCATOR_CURSOR).loadAddress();
        Address limit = allocatorBase.plus(BUMP_ALLOCATOR_LIMIT).loadAddress();
        Offset delta = negOff.minus(cursor.toWord()).and(mask).toOffset();
        Address result = cursor.plus(delta);
        Address newCursor = result.plus(bytes);

        // VM.sysWrite("fast alloc: cursor = ", cursor); VM.sysWrite(", aligned to ", result); VM.sysWriteln(", limit = ", limit);
        // sysCall.sysConsoleFlushErrorAndTrace();

        if (newCursor.GT(limit)) {
            return slowPath(bytes, align, offset, allocator);
            
            // return sysCall.sysAllocSlowBumpMonotoneImmortal(allocatorBase, bytes, align, offset, allocator);
            // return sysCall.sysAlloc(Magic.objectAsAddress(this).plus(MUTATOR_BASE_OFFSET), bytes, align, offset, allocator);
        } else {
            // VM.sysWriteln("return ", result);
            // VM.sysWriteln("save new cursor ", newCursor);
            // sysCall.sysConsoleFlushErrorAndTrace();
            allocatorBase.plus(BUMP_ALLOCATOR_CURSOR).store(newCursor);
            return result;
        }        
    }

    @NoInline
    protected final Address slowPath(int bytes, int align, int offset, int allocator) {
        // VM.sysWriteln("======go to slow alloc");
        // sysCall.sysConsoleFlushErrorAndTrace();
        Address handle = Magic.objectAsAddress(this).plus(MUTATOR_BASE_OFFSET);
        return sysCall.sysAlloc(handle, bytes, align, offset, allocator);
    }

    @Override
    @Inline
    public void postAlloc(ObjectReference ref, ObjectReference typeRef,
                          int bytes, int allocator) {
        allocator = mapAllocator(bytes, allocator);
        int space = getSpaceTag(allocator);

        if (space == COPY_SPACE) {
            // nothing to do for post_alloc
        } else if (space == IMMORTAL_SPACE || space == LARGE_OBJECT_SPACE) {
            Address handle = Magic.objectAsAddress(this).plus(MUTATOR_BASE_OFFSET);
            sysCall.sysPostAlloc(handle, ref, typeRef, bytes, allocator);
        }
    }

    public Address setBlock(Address mmtkHandle) {
        // copy MUTATOR_SIZE's words from mmtkHandle to (this+MUTATOR_BASE_OFFSET)
        Address src = mmtkHandle;
        Address dst = Magic.objectAsAddress(this).plus(MUTATOR_BASE_OFFSET);

        // VM.sysWrite("setBlock() copy from "); VM.sysWrite(src); VM.sysWrite(" to "); VM.sysWriteln(dst);

        // copy word by word
        for (int offset = 0; offset < MUTATOR_SIZE; offset += 4) {
            Address srcAddr = src.plus(offset);
            Address dstAddr = dst.plus(offset);
            Word val = srcAddr.loadWord();
            
            // VM.sysWrite("Copying "); VM.sysWrite(val); VM.sysWrite(" from 0x"); VM.sysWrite(srcAddr);
            // VM.sysWrite(" to "); VM.sysWriteln(dstAddr);
            
            dstAddr.store(val);
        }
        // sysCall.sysConsoleFlushErrorAndTrace();

        return dst;
    }    

    public void collectionPhase(short phaseId, boolean primary) {
        VM.sysFail("unreachable");
    }
}