use mmtk::util::opaque_pointer::*;
use mmtk::util::ObjectReference;
use mmtk::vm::ReferenceGlue;

use JikesRVM;

pub struct VMReferenceGlue {}

impl ReferenceGlue<JikesRVM> for VMReferenceGlue {
    type FinalizableType = ObjectReference;

    fn set_referent(_reff: ObjectReference, _referent: ObjectReference) {
        unimplemented!()
    }

    fn get_referent(_object: ObjectReference) -> ObjectReference {
        unimplemented!()
    }

    fn enqueue_references(_references: &[ObjectReference], _tls: VMWorkerThread) {
        unimplemented!()
    }
}
