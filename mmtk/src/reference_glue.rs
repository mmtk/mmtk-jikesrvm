use mmtk::util::opaque_pointer::*;
use mmtk::util::ObjectReference;
use mmtk::vm::ReferenceGlue;

use entrypoint::*;
use JikesRVM;

use std::arch::asm;

pub struct VMReferenceGlue {}

impl ReferenceGlue<JikesRVM> for VMReferenceGlue {
    fn set_referent(reff: ObjectReference, referent: ObjectReference) {
        unsafe {
            (reff.to_address() + REFERENCE_REFERENT_FIELD_OFFSET).store(referent.value());
        }
    }

    fn get_referent(object: ObjectReference) -> ObjectReference {
        debug_assert!(!object.is_null());
        unsafe { (object.to_address() + REFERENCE_REFERENT_FIELD_OFFSET).load::<ObjectReference>() }
    }

    fn enqueue_references(references: &[ObjectReference], tls: VMWorkerThread) {
        for reff in references {
            unsafe {
                jtoc_call!(
                    ENQUEUE_REFERENCE_METHOD_OFFSET,
                    tls,
                    std::mem::transmute::<_, usize>(*reff)
                );
            }
        }
    }
}
