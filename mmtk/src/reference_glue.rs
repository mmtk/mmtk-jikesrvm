use mmtk::util::opaque_pointer::*;
use mmtk::util::ObjectReference;
use mmtk::vm::ReferenceGlue;

#[cfg(not(feature = "binding_side_ref_proc"))]
use entrypoint::*;

use JikesRVM;

#[cfg(not(feature = "binding_side_ref_proc"))]
use std::arch::asm;

pub struct VMReferenceGlue {}

impl ReferenceGlue<JikesRVM> for VMReferenceGlue {
    type FinalizableType = ObjectReference;

    fn set_referent(reff: ObjectReference, referent: ObjectReference) {
        if cfg!(feature = "binding_side_ref_proc") {
            panic!();
        } else {
            unsafe {
                (reff.to_raw_address() + REFERENCE_REFERENT_FIELD_OFFSET).store(referent);
            }
        }
    }

    fn get_referent(object: ObjectReference) -> ObjectReference {
        if cfg!(feature = "binding_side_ref_proc") {
            panic!();
        } else {
            debug_assert!(!object.is_null());
            unsafe {
                (object.to_raw_address() + REFERENCE_REFERENT_FIELD_OFFSET)
                    .load::<ObjectReference>()
            }
        }
    }

    fn enqueue_references(references: &[ObjectReference], tls: VMWorkerThread) {
        if cfg!(feature = "binding_side_ref_proc") {
            panic!();
        } else {
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
}
