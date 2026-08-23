package org.wickra.internal;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Locale;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

/**
 * The loader used to take the first file it found named like the native library
 * and abort if that one did not export the Wickra ABI, so a stale or unrelated
 * build earlier in the search order hid a good one further up. It collects every
 * candidate now and tries them in order.
 */
class NativeResolutionTest {

    private static String libraryFileName() {
        String os = System.getProperty("os.name", "").toLowerCase(Locale.ROOT);
        if (os.contains("win")) {
            return "wickra.dll";
        }
        return (os.contains("mac") || os.contains("darwin")) ? "libwickra.dylib" : "libwickra.so";
    }

    private static Path stage(Path root, String profile) throws IOException {
        Path dir = root.resolve("target").resolve(profile);
        Files.createDirectories(dir);
        Path lib = dir.resolve(libraryFileName());
        Files.writeString(lib, "not a real library");
        return lib;
    }

    @Test
    void collectsEveryCandidateNotJustTheFirst(@TempDir Path tmp) throws IOException {
        Path outer = tmp.resolve("outer");
        Path inner = outer.resolve("inner");
        Files.createDirectories(inner);
        Path outerLib = stage(outer, "release");
        Path innerLib = stage(inner, "release");

        List<Path> found = WickraNative.findInCargoTarget(new Path[] {inner});

        assertEquals(List.of(innerLib, outerLib), found,
                "both candidates should be offered, nearest first");
    }

    @Test
    void prefersReleaseOverDebugWithinOneDirectory(@TempDir Path tmp) throws IOException {
        Path release = stage(tmp, "release");
        Path debug = stage(tmp, "debug");

        List<Path> found = WickraNative.findInCargoTarget(new Path[] {tmp});

        assertEquals(List.of(release, debug), found);
    }

    @Test
    void reportsNothingWhenThereIsNothingToFind(@TempDir Path tmp) {
        assertTrue(WickraNative.findInCargoTarget(new Path[] {tmp}).isEmpty());
    }

    @Test
    void skipsNullBases(@TempDir Path tmp) throws IOException {
        Path lib = stage(tmp, "release");
        assertEquals(List.of(lib), WickraNative.findInCargoTarget(new Path[] {null, tmp}));
    }
}
