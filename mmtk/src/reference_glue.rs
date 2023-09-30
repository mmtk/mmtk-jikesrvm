use mmtk::util::opaque_pointer::*;
use mmtk::util::ObjectReference;
use mmtk::vm::ReferenceGlue;

use JikesRVM;

pub struct VMReferenceGlue {}

impl ReferenceGlue<JikesRVM> for VMReferenceGlue {
    type FinalizableType = ObjectReference;

    fn set_referent(reff: ObjectReference, referent: ObjectReference) {
        unimplemented!()
    }

    fn get_referent(object: ObjectReference) -> ObjectReference {
        unimplemented!()
    }

    fn enqueue_references(references: &[ObjectReference], tls: VMWorkerThread) {
        unimplemented!()
    }
}
