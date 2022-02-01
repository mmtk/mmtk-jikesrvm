#ifndef MMTK_H
#define MMTK_H

#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef void* MMTk_Mutator;
typedef void* MMTk_TraceLocal;

/**
 * Allocation
 */
extern MMTk_Mutator bind_mutator(void *tls);
extern void destroy_mutator(MMTk_Mutator mutator);

extern void* alloc(MMTk_Mutator mutator, size_t size,
    size_t align, size_t offset, int allocator);

extern void* alloc_slow_bump_monotone_immortal(MMTk_Mutator mutator, size_t size,
    size_t align, size_t offset, int allocator);
extern void* alloc_slow_bump_monotone_copy(MMTk_Mutator mutator, size_t size,
    size_t align, size_t offset, int allocator);
extern void* alloc_slow_largeobject(MMTk_Mutator mutator, size_t size,
    size_t align, size_t offset, int allocator);

extern void post_alloc(MMTk_Mutator mutator, void* refer, void* type_refer,
    int bytes, int allocator);

extern bool is_live_object(void* obj);
extern bool is_mapped_object(void* obj);
extern bool is_mapped_address(void* addr);
extern void modify_check(void* ref);

/**
 * Misc
 */
extern void gc_init(size_t heap_size);
extern bool will_never_move(void* object);
extern bool process(char* name, char* value);
extern void scan_region();
extern void handle_user_collection_request(void *tls);

extern void start_control_collector(void *tls, void* controller);
extern void start_worker(void *tls, void* worker);

extern void release_buffer(void* buffer);

/**
 * JikesRVM-specific
 */
extern void jikesrvm_gc_init(void* jtoc, size_t heap_size);

extern void enable_collection(void *tls);

// For the following functions, glue.asm provides a wrapper with JikesRVM's calling convention
// to call the internal functions.

extern void* jikesrvm_alloc(MMTk_Mutator mutator, size_t size,
    size_t align, size_t offset, int allocator);

extern void* jikesrvm_alloc_slow_bump_monotone_immortal(MMTk_Mutator mutator, size_t size,
    size_t align, size_t offset, int allocator);
extern void* jikesrvm_alloc_slow_bump_monotone_copy(MMTk_Mutator mutator, size_t size,
    size_t align, size_t offset, int allocator);
extern void* jikesrvm_alloc_slow_largeobject(MMTk_Mutator mutator, size_t size,
    size_t align, size_t offset, int allocator);

extern void jikesrvm_handle_user_collection_request(void *tls);

extern void jikesrvm_harness_begin(void *tls);

/**
 * VM Accounting
 */
extern size_t free_bytes();
extern size_t total_bytes();
extern size_t used_bytes();
extern void* starting_heap_address();
extern void* last_heap_address();

//  Last_gc_time();

/**
 * Reference Processing
 */
extern void add_weak_candidate(void* ref, void* referent);
extern void add_soft_candidate(void* ref, void* referent);
extern void add_phantom_candidate(void* ref, void* referent);

/**
 * Finalization
 */
extern void add_finalizer(void* obj);
extern void* get_finalized_object();

extern void harness_begin(void *tls);
extern void harness_end();

#ifdef __cplusplus
}
#endif

#endif // MMTK_H