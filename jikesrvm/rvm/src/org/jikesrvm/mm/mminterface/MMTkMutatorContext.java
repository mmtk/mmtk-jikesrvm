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
    // Due to the fact that RJava does not support unboxed struct or nested struct, we have to layout all the fields here.

    // Mutator section starts

    // Allocators
    // 6 x BumpAllocator (6 x 6 words)

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
    @Entrypoint
    Address bumpAllocator0PlanFat;

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
    @Entrypoint
    Address bumpAllocator1PlanFat;

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
    @Entrypoint
    Address bumpAllocator2PlanFat;

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
    @Entrypoint
    Address bumpAllocator3PlanFat;

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
    @Entrypoint
    Address bumpAllocator4PlanFat;

    // Bump Allocator 5
    @Entrypoint
    Address bumpAllocator5Tls;
    @Entrypoint
    Address bumpAllocator5Cursor;
    @Entrypoint
    Address bumpAllocator5Limit;
    @Entrypoint
    Address bumpAllocator5Space;
    @Entrypoint
    Address bumpAllocator5SpaceFat;
    @Entrypoint
    Address bumpAllocator5Plan;
    @Entrypoint
    Address bumpAllocator5PlanFat;

    // 2 x LargeObjectAllocator (1 x 4 words)
    @Entrypoint
    Address largeObjectAllocator0Tls;
    @Entrypoint
    Address largeObjectAllocator0Space;
    @Entrypoint
    Address largeObjectAllocator0Plan;
    @Entrypoint
    Address largeObjectAllocator0PlanFat;

    @Entrypoint
    Address largeObjectAllocator1Tls;
    @Entrypoint
    Address largeObjectAllocator1Space;
    @Entrypoint
    Address largeObjectAllocator1Plan;
    @Entrypoint
    Address largeObjectAllocator1PlanFat;

    // 1 x MallocAllocator
    @Entrypoint
    Address mallocAllocator0Tls;
    @Entrypoint
    Address mallocAllocator0Space;
    @Entrypoint
    Address mallocAllocator0Plan;
    @Entrypoint
    Address mallocAllocator0PlanFat;

    // 1 x ImmixAllocator
    @Entrypoint
    Address immixAllocator0Tls;
    @Entrypoint
    Address immixAllocator0Cursor;
    @Entrypoint
    Address immixAllocator0Limit;
    @Entrypoint
    Address immixAllocator0Space;
    @Entrypoint
    Address immixAllocator0Plan;
    @Entrypoint
    Address immixAllocator0PlanFat;
    @Entrypoint
    Address immixAllocator0Hot;
    @Entrypoint
    Address immixAllocator0Copy;
    @Entrypoint
    Address immixAllocator0LargeCursor;
    @Entrypoint
    Address immixAllocator0LargeLimit;
    @Entrypoint
    Address immixAllocator0RequestForLarge;
    @Entrypoint
    Address immixAllocator0OptionLineTag;
    @Entrypoint
    Address immixAllocator0OptionLineData;

    // 1 x MarkCompactAllocator (same layout as bump allocator)
    @Entrypoint
    Address markCompactAllocator0Tls;
    @Entrypoint
    Address markCompactAllocator0Cursor;
    @Entrypoint
    Address markCompactAllocator0Limit;
    @Entrypoint
    Address markCompactAllocator0Space;
    @Entrypoint
    Address markCompactAllocator0SpaceFat;
    @Entrypoint
    Address markCompactAllocator0Plan;
    @Entrypoint
    Address markCompactAllocator0PlanFat;

    // barrier
    @Entrypoint
    Address barrier;
    Address barrier_fat;

    // mutator_tls
    @Entrypoint
    Address mutatorTls;
    // plan
    @Entrypoint
    Address plan;
    @Entrypoint
    Address planFat;

    // MutatorConfig
    // allocator_mapping
    @Entrypoint
    Address allocatorMapping;
    // space_mapping
    @Entrypoint
    Address spaceMapping;
    // collection_phase_func (fat pointer)
    @Entrypoint
    Address prepare_func;
    Address prepare_func_fat;
    @Entrypoint
    Address release_func;
    Address release_func_fat;

    @Entrypoint
    Address __mutatorDataEnd;

    // Mutator section ends

    // Number of allocators - these constants need to match the layout of the fields, also the constants in MMTk core.
    static final int MAX_BUMP_ALLOCATORS = 6;
    static final int MAX_LARGE_OBJECT_ALLOCATORS = 2;
    static final int MAX_MALLOC_ALLOCATORS = 1;
    static final int MAX_IMMIX_ALLOCATORS = 1;
    static final int MAX_MARK_COMPACT_ALLOCATORS = 1;
    // Bump allocator size
    static final int BUMP_ALLOCATOR_SIZE = 7 * BYTES_IN_WORD;
    // Bump allocator field offsets
    static final int BUMP_ALLOCATOR_TLS = 0;
    static final int BUMP_ALLOCATOR_CURSOR = BYTES_IN_WORD;
    static final int BUMP_ALLOCATOR_LIMIT = BYTES_IN_WORD * 2;
    static final int BUMP_ALLOCATOR_SPACE = BYTES_IN_WORD * 3;
    static final int BUMP_ALLOCATOR_SPACE_FAT = BYTES_IN_WORD * 4;
    static final int BUMP_ALLOCATOR_PLAN = BYTES_IN_WORD * 5;
    static final int BUMP_ALLOCATOR_PLAN_FAT = BYTES_IN_WORD * 6;
    // Large object allocator size. We do not need offsets for each field, as we don't need to implement fastpath for large object allocator.
    static final int LARGE_OBJECT_ALLOCATOR_SIZE = 4 * BYTES_IN_WORD;
    // Malloc allocator size. We do not need offsets for each field, as we don't need to implement fastpath for large object allocator.
    static final int MALLOC_ALLOCATOR_SIZE = 4 * BYTES_IN_WORD;
    // Immix allocator size
    static final int IMMIX_ALLOCATOR_SIZE = 13 * BYTES_IN_WORD;
    // Mark compact allocator size (the same as bump allocator)
    static final int MARK_COMPACT_ALLOCATOR_SIZE = BUMP_ALLOCATOR_SIZE;

    // The base offset of this mutator section
    static final Offset MUTATOR_BASE_OFFSET = EntrypointHelper.getField(MMTkMutatorContext.class, "bumpAllocator0Tls", Address.class).getOffset();
    static final Offset BUMP_ALLOCATOR_OFFSET = EntrypointHelper.getField(MMTkMutatorContext.class, "bumpAllocator0Tls", Address.class).getOffset();
    static final Offset LARGE_OBJECT_ALLOCATOR_OFFSET = EntrypointHelper.getField(MMTkMutatorContext.class, "largeObjectAllocator0Tls", Address.class).getOffset();
    static final Offset MALLOC_ALLOCATOR_OFFSET = EntrypointHelper.getField(MMTkMutatorContext.class, "mallocAllocator0Tls", Address.class).getOffset();
    static final Offset MUTATOR_DATA_END_OFFSET = EntrypointHelper.getField(MMTkMutatorContext.class, "__mutatorDataEnd", Address.class).getOffset();

    // The size of this mutator section
    static final int MUTATOR_SIZE = MUTATOR_DATA_END_OFFSET.toInt() - MUTATOR_BASE_OFFSET.toInt();

    // tag for allocator type
    public static final int TAG_BUMP_POINTER = 0;
    public static final int TAG_LARGE_OBJECT = 1;
    public static final int TAG_MALLOC = 2;
    public static final int TAG_IMMIX = 3;
    public static final int TAG_MARK_COMPACT = 4;

    // tag for space type
    public static final int IMMORTAL_SPACE = 0;
    public static final int COPY_SPACE = 1;
    public static final int LARGE_OBJECT_SPACE = 2;
    public static final int MALLOC_SPACE = 3;

    // The implementation of these methods are per plan, and they should match the allocatorMapping and spaceMapping in MMTk core for a plan.
    // The reason we need to reimplement them in the fastpath is that we need the compiler to see the code and be able to do constant propagation
    // and optimize the branches away. The implementation of these method should be simple to help constant propagation.
    @Inline
    protected abstract int getAllocatorTag(int allocator);
    @Inline
    protected abstract int getAllocatorIndex(int allocator);
    @Inline
    protected abstract int getSpaceTag(int allocator);

    // Mapping JikesRVM's allocator to MMTk's allocation semantic. This should get optimized away by the opt compiler.
    @Inline
    public final int mapAllocator(int bytes, int origAllocator) {
        if (origAllocator == Plan.ALLOC_DEFAULT)
            return MMTkAllocator.DEFAULT;
        else if (origAllocator == Plan.ALLOC_IMMORTAL)
            return MMTkAllocator.IMMORTAL;
        else if (origAllocator == Plan.ALLOC_LOS)
            return MMTkAllocator.LOS;
        else if (origAllocator == Plan.ALLOC_CODE || origAllocator == Plan.ALLOC_LARGE_CODE)
            return MMTkAllocator.CODE;
        else {
            return MMTkAllocator.IMMORTAL;
        }
    }

    // Allocation fastpath. Most of the branches should be optimized away by the opt compiler.
    @Override
    @Inline
    public final Address alloc(int bytes, int align, int offset, int allocator, int site) {
        allocator = mapAllocator(bytes, allocator);

        // Each plan will return which allocator to use
        int ty = getAllocatorTag(allocator);
        int index = getAllocatorIndex(allocator);

        if (ty == TAG_BUMP_POINTER) {
            return bumpAllocatorFastPath(bytes, align, offset, allocator, index);
        } else if (ty == TAG_LARGE_OBJECT || ty == TAG_MALLOC) {
            // No fastpath for large object allocator. We just use the general slowpath.
            return slowPath(bytes, align, offset, allocator);
        } else {
            VM.sysFail("unimplemented");
            return Address.zero();
        }
    }

    // Allocation fastpath for bump pointer allocator.
    // This fastpath works for any of the bump allocators in the mutator, specified by the allocatorIndex.
    // As a result, we cannot directly refer to the fields of each bump allocator by field names.
    // Instead, we refer to the fields by calculated offsets from BUMP_ALLOCATOR_OFFSET.
    // A consequence is that the opt compiler does not generate optimal machine code for accessing those fields by offsets, and this fastpath
    // is slightly slower than the original one (roughtly 2 more LEAs on x86). If this is a concern, we can duplicate this fastpath implementation for
    // each bump allocator, and each duplication maps to one bump allocator. In this case, we can refer to the fields directly in each dupilcation,
    // and the performance should be the same as before. But the code is less maintainable.
    @Inline
    protected final Address bumpAllocatorFastPath(int bytes, int align, int offset, int allocator, int allocatorIndex) {
        // Align allocation
        Word mask = Word.fromIntSignExtend(align - 1);
        Word negOff = Word.fromIntSignExtend(-offset);

        // allocatorIndex should be compile-time constant. We do not need to worry about the multiplication.
        // The offset will be optimized to a constant.
        Address allocatorBase = Magic.objectAsAddress(this).plus(BUMP_ALLOCATOR_OFFSET).plus(allocatorIndex * BUMP_ALLOCATOR_SIZE);

        Address cursor = allocatorBase.plus(BUMP_ALLOCATOR_CURSOR).loadAddress();
        Address limit = allocatorBase.plus(BUMP_ALLOCATOR_LIMIT).loadAddress();
        Offset delta = negOff.minus(cursor.toWord()).and(mask).toOffset();
        Address result = cursor.plus(delta);
        Address newCursor = result.plus(bytes);

        if (newCursor.GT(limit)) {
            // Out of local buffer, use the general slowpath
            return slowPath(bytes, align, offset, allocator);
        } else {
            // Save the new cursor
            allocatorBase.plus(BUMP_ALLOCATOR_CURSOR).store(newCursor);
            return result;
        }
    }

    // General allocation slowpath.
    @NoInline
    protected final Address slowPath(int bytes, int align, int offset, int allocator) {
        Address handle = Magic.objectAsAddress(this).plus(MUTATOR_BASE_OFFSET);
        return sysCall.sysAlloc(handle, bytes, align, offset, allocator);
    }

    // Post allocation fastpath. Note this is not completely implemented.
    @Override
    @Inline
    public void postAlloc(ObjectReference ref, ObjectReference typeRef, int bytes, int allocator) {
        allocator = mapAllocator(bytes, allocator);
        // It depends on the space to decide which fastpath we should use
        int space = getSpaceTag(allocator);

        if (space == COPY_SPACE) {
            // Nothing to do for post_alloc for CopySpace
        } else {
            // slowpath to call into MMTk core's post_alloc()
            // TODO: We should further implement fastpaths for certain spaces, such as ImmortalSpace.
            Address handle = Magic.objectAsAddress(this).plus(MUTATOR_BASE_OFFSET);
            sysCall.sysPostAlloc(handle, ref, typeRef, bytes, allocator);
        }
    }

    public Address setBlock(Address mmtkHandle) {
        // Copy MUTATOR_SIZE's bytes from mmtkHandle to (this + MUTATOR_BASE_OFFSET)
        Address src = mmtkHandle;
        Address dst = Magic.objectAsAddress(this).plus(MUTATOR_BASE_OFFSET);

        // copy word by word
        for (int offset = 0; offset < MUTATOR_SIZE; offset += 4) {
            Address srcAddr = src.plus(offset);
            Address dstAddr = dst.plus(offset);
            dstAddr.store(srcAddr.loadWord());
        }

        return dst;
    }

    public void collectionPhase(short phaseId, boolean primary) {
        VM.sysFail("unreachable");
    }
}