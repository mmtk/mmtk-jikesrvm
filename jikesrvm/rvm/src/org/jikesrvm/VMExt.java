package org.jikesrvm;

import org.jikesrvm.scheduler.RVMThread;
import org.vmmagic.pragma.Entrypoint;

public class VMExt {
    @Entrypoint
    public static int currentThreadSwitchTo(int newStatus) {
        RVMThread cur = RVMThread.getCurrentThread();
        int old = cur.getExecStatus();
        cur.setExecStatus(newStatus);
        return old;
    }
}
