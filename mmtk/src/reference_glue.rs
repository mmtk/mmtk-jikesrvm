use mmtk::util::opaque_pointer::*;
use mmtk::util::ObjectReference;
use mmtk::vm::ReferenceGlue;

use crate::JikesRVM;

use std::convert::TryInto;

use crate::jikesrvm_calls;
use crate::object_model::JikesObj;

pub struct VMReferenceGlue {}

impl ReferenceGlue<JikesRVM> for VMReferenceGlue {
    type FinalizableType = ObjectReference;

    fn set_referent(reff: ObjectReference, referent: ObjectReference) {
        if cfg!(feature = "binding_side_ref_proc") {
            panic!();
        }

        JikesObj::from(reff).set_referent(JikesObj::from(referent))
    }

    fn get_referent(reff: ObjectReference) -> Option<ObjectReference> {
        if cfg!(feature = "binding_side_ref_proc") {
            panic!();
        }

        JikesObj::from(reff).get_referent().try_into().ok()
    }

    fn enqueue_references(references: &[ObjectReference], tls: VMWorkerThread) {
        if cfg!(feature = "binding_side_ref_proc") {
            panic!();
        } else {
            for reff in references {
                let jikes_reff = JikesObj::from(*reff);
                jikesrvm_calls::enqueue_reference(tls, jikes_reff);
            }
        }
    }

    fn clear_referent(new_reference: ObjectReference) {
        if cfg!(feature = "binding_side_ref_proc") {
            panic!();
        } else {
            JikesObj::from(new_reference).set_referent(JikesObj::NULL)
        }
    }
}
