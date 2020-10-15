package org.jikesrvm.mm.mminterface;

// The values need to match the values of enum Allocator in plan.rs
public class MMTkAllocator {
    public static final int DEFAULT = 0;
    public static final int IMMORTAL = 1;
    public static final int LOS = 2;
    public static final int CODE = 3;
    public static final int READONLY = 4;
}
