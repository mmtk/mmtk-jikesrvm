/*
 *  This file is part of the Jikes RVM project (http://jikesrvm.org).
 *
 *  This file is licensed to You under the Eclipse Public License (EPL);
 *  You may not use this file except in compliance with the License. You
 *  may obtain a copy of the License at
 *
 *      http://www.opensource.org/licenses/eclipse-1.0.php
 *
 *  See the COPYRIGHT.txt file distributed with this work for information
 *  regarding copyright ownership.
 */
package org.jikesrvm.mm.mminterface;

import org.jikesrvm.VM;
import org.jikesrvm.mm.mmtk.ScanBootImage;
import org.jikesrvm.mm.mmtk.ScanThread;
import org.jikesrvm.runtime.BootRecord;
import org.mmtk.utility.Log;
import org.vmmagic.pragma.Entrypoint;
import org.vmmagic.pragma.Inline;
import org.vmmagic.pragma.Uninterruptible;
import org.vmmagic.unboxed.Address;
import org.vmmagic.unboxed.Offset;

import static org.jikesrvm.mm.mmtk.ScanBootImage.checkReference;
import static org.jikesrvm.mm.mmtk.ScanBootImage.isAddressAligned;
import static org.mmtk.utility.Constants.BYTES_IN_ADDRESS;

public class RustScanning {

    private static final boolean DEBUG = false;
    private static final boolean FILTER = true;

    private static final int LOG_CHUNK_BYTES = 12;
    private static final int LONGENCODING_MASK = 0x1;
    private static final int RUN_MASK = 0x2;
    private static final int LONGENCODING_OFFSET_BYTES = 4;
    private static final int ADDRESS_LENGTH = BYTES_IN_ADDRESS * 8;



    /* statistics */
    static int roots = 0;
    static int refs = 0;

    @Inline
    @Uninterruptible
    @Entrypoint
    public static void scanBootImage(Address root) {
        /* establish sentinels in map & image */
        Address mapStart = BootRecord.the_boot_record.bootImageRMapStart;
        Address mapEnd = BootRecord.the_boot_record.bootImageRMapEnd;
        Address imageStart = BootRecord.the_boot_record.bootImageDataStart;

        /* establish roots */
        Address rootCursor = root;

        /* figure out striding */
        int stride = 1 << LOG_CHUNK_BYTES;
        int start = 0 << LOG_CHUNK_BYTES;
        Address cursor = mapStart.plus(start);

        /* statistics */
        roots = 0;
        refs = 0;

        /* process chunks in parallel till done */
        while (cursor.LT(mapEnd)) {
            rootCursor = processChunk(cursor, imageStart, mapStart, mapEnd, rootCursor);
            cursor = cursor.plus(stride);
        }

        /* print some debugging stats */
        Log.write("<boot image");
        Log.write(" roots: ", roots);
        Log.write(" refs: ", refs);
        Log.write(">");
    }

    @Inline
    @Uninterruptible
    static Address processChunk(Address chunkStart, Address imageStart,
                                    Address mapStart, Address mapEnd, Address rootCursor) {
        int value;
        Offset offset = Offset.zero();
        Address cursor = chunkStart;
        while ((value = (cursor.loadByte() & 0xff)) != 0) {
            /* establish the offset */
            if ((value & LONGENCODING_MASK) != 0) {
                offset = ScanBootImage.decodeLongEncoding(cursor);
                cursor = cursor.plus(LONGENCODING_OFFSET_BYTES);
            } else {
                offset = offset.plus(value & 0xfc);
                cursor = cursor.plus(1);
            }
            /* figure out the length of the run, if any */
            int runlength = 0;
            if ((value & RUN_MASK) != 0) {
                runlength = cursor.loadByte() & 0xff;
                cursor = cursor.plus(1);
            }
            /* enqueue the specified slot or slots */
            if (VM.VerifyAssertions) VM._assert(isAddressAligned(offset));
            Address slot = imageStart.plus(offset);
            if (DEBUG) refs++;
            if (!FILTER || slot.loadAddress().GT(mapEnd)) {
                if (DEBUG) roots++;
                rootCursor.store(slot);
                rootCursor.plus(ADDRESS_LENGTH);
                //trace.processRootEdge(slot, false);
            }
            if (runlength != 0) {
                for (int i = 0; i < runlength; i++) {
                    offset = offset.plus(BYTES_IN_ADDRESS);
                    slot = imageStart.plus(offset);
                    if (VM.VerifyAssertions) VM._assert(isAddressAligned(slot));
                    if (DEBUG) refs++;
                    if (!FILTER || slot.loadAddress().GT(mapEnd)) {
                        if (DEBUG) roots++;
                        if (ScanThread.VALIDATE_REFS) checkReference(slot);
                        rootCursor.store(slot);
                        rootCursor.plus(ADDRESS_LENGTH);
                        //trace.processRootEdge(slot, false);
                    }
                }
            }
        }
        return rootCursor;
    }

}
