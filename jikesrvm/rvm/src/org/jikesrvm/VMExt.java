package org.jikesrvm;

import org.jikesrvm.scheduler.RVMThread;
import org.vmmagic.pragma.Entrypoint;

public class VMExt {
    @Entrypoint
    public static int enterVM() {
        RVMThread cur = RVMThread.getCurrentThread();
        int old = cur.getExecStatus();
        cur.setExecStatus(RVMThread.IN_JAVA);
        return old;
    }

    @Entrypoint
    public static void leaveVM(int status) {
        RVMThread cur = RVMThread.getCurrentThread();
        cur.setExecStatus(status);
    }
}
