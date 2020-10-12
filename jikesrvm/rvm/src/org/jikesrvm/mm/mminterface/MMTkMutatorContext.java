package org.jikesrvm.mm.mminterface;

import org.vmmagic.pragma.*;
import org.vmmagic.unboxed.*;
import org.mmtk.plan.MutatorContext;
import org.jikesrvm.VM;

@Uninterruptible
public class MMTkMutatorContext extends MutatorContext{
    public void collectionPhase(short phaseId, boolean primary) {
        VM.sysFail("unreachable");
    }
}