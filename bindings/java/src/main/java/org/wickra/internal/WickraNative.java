package org.wickra.internal;

import java.io.IOException;
import java.io.InputStream;
import java.lang.foreign.Arena;
import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.invoke.MethodHandle;
import java.lang.ref.Cleaner;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.nio.file.StandardCopyOption;
import java.util.Locale;

/**
 * Native library resolution and FFM downcall plumbing for the Wickra C ABI.
 *
 * <p>The native library is located in one of two ways. When the binding is
 * consumed as a packaged jar the per-platform library ships under
 * {@code /native/<os>-<arch>/} and is extracted to a temporary file at load
 * time. For local development (running against a {@code cargo build}) the
 * resolver walks up the directory tree to find {@code target/release} or
 * {@code target/debug}. Every candidate is validated to actually export the
 * Wickra ABI before it is accepted, so an unrelated library of the same name
 * cannot shadow the real one.
 *
 * <p>This is internal plumbing; application code uses the generated indicator
 * classes in {@code org.wickra}.
 */
public final class WickraNative {
    private WickraNative() {
    }

    /** Any exported symbol works as a fingerprint; sma_new exists in every build. */
    private static final String SENTINEL = "wickra_sma_new";

    static final Cleaner CLEANER = Cleaner.create();
    private static final Linker LINKER = Linker.nativeLinker();
    private static final Arena LIB_ARENA = Arena.ofShared();
    private static final SymbolLookup LOOKUP = loadLibrary();

    /** Build a downcall handle for one C function. Internal use by the generated code. */
    public static MethodHandle downcall(String name, FunctionDescriptor descriptor) {
        MemorySegment symbol = LOOKUP.find(name)
                .orElseThrow(() -> new UnsatisfiedLinkError("wickra: missing symbol " + name));
        return LINKER.downcallHandle(symbol, descriptor);
    }

    /**
     * Register an opaque handle for release via its {@code _free} function when the
     * owning wrapper becomes unreachable (or is closed). The action holds no
     * reference to the owner, so it never keeps it alive.
     */
    public static Cleaner.Cleanable register(Object owner, MemorySegment handle, MethodHandle free) {
        return CLEANER.register(owner, new FreeAction(handle, free));
    }

    /**
     * Allocate a C {@code bool*} buffer (one byte per element) from flag values.
     * The C ABI takes the cross-section state flags as {@code const bool*}, so
     * they must be one byte each rather than eight-byte doubles.
     */
    public static MemorySegment boolSegment(Arena arena, boolean[] flags) {
        byte[] bytes = new byte[flags.length];
        for (int i = 0; i < flags.length; i++) {
            bytes[i] = (byte) (flags[i] ? 1 : 0);
        }
        return arena.allocateFrom(java.lang.foreign.ValueLayout.JAVA_BYTE, bytes);
    }

    /** Re-throw a {@link MethodHandle#invokeExact} {@link Throwable} as an unchecked exception. */
    public static RuntimeException rethrow(Throwable t) {
        if (t instanceof RuntimeException re) {
            return re;
        }
        if (t instanceof Error e) {
            throw e;
        }
        return new RuntimeException(t);
    }

    private record FreeAction(MemorySegment handle, MethodHandle free) implements Runnable {
        @Override
        public void run() {
            try {
                free.invokeExact(handle);
            } catch (Throwable ignored) {
                // Best-effort release during finalization; nothing actionable here.
            }
        }
    }

    private static SymbolLookup loadLibrary() {
        Path lib = locate();
        if (lib == null) {
            throw new UnsatisfiedLinkError(
                    "wickra: could not locate the native library (" + libraryFileName()
                            + "). Bundle it under resources/native/" + platformDir()
                            + "/ or build the C ABI with `cargo build -p wickra-c --release`.");
        }
        SymbolLookup lookup = SymbolLookup.libraryLookup(lib, LIB_ARENA);
        if (lookup.find(SENTINEL).isEmpty()) {
            throw new UnsatisfiedLinkError("wickra: " + lib + " does not export the C ABI");
        }
        return lookup;
    }

    private static Path locate() {
        Path bundled = extractBundled();
        if (bundled != null) {
            return bundled;
        }
        return findInCargoTarget();
    }

    private static Path extractBundled() {
        String resource = "/native/" + platformDir() + "/" + libraryFileName();
        try (InputStream in = WickraNative.class.getResourceAsStream(resource)) {
            if (in == null) {
                return null;
            }
            Path tmp = Files.createTempFile("wickra-", "-" + libraryFileName());
            tmp.toFile().deleteOnExit();
            Files.copy(in, tmp, StandardCopyOption.REPLACE_EXISTING);
            return tmp;
        } catch (IOException e) {
            return null;
        }
    }

    private static Path findInCargoTarget() {
        String file = libraryFileName();
        Path[] bases = {Paths.get(System.getProperty("user.dir", ".")), codeSourceDir()};
        for (Path base : bases) {
            Path dir = base;
            for (int i = 0; i < 16 && dir != null; i++) {
                for (String profile : new String[] {"release", "debug"}) {
                    Path candidate = dir.resolve("target").resolve(profile).resolve(file);
                    if (Files.isRegularFile(candidate)) {
                        return candidate;
                    }
                }
                dir = dir.getParent();
            }
        }
        return null;
    }

    private static Path codeSourceDir() {
        try {
            Path p = Paths.get(WickraNative.class.getProtectionDomain()
                    .getCodeSource().getLocation().toURI());
            return Files.isDirectory(p) ? p : p.getParent();
        } catch (Exception e) {
            return null;
        }
    }

    private static String osName() {
        String os = System.getProperty("os.name", "").toLowerCase(Locale.ROOT);
        if (os.contains("win")) {
            return "win";
        }
        if (os.contains("mac") || os.contains("darwin")) {
            return "osx";
        }
        return "linux";
    }

    private static String archName() {
        String arch = System.getProperty("os.arch", "").toLowerCase(Locale.ROOT);
        if (arch.equals("aarch64") || arch.equals("arm64")) {
            return "arm64";
        }
        return "x64";
    }

    private static String platformDir() {
        return osName() + "-" + archName();
    }

    private static String libraryFileName() {
        return switch (osName()) {
            case "win" -> "wickra.dll";
            case "osx" -> "libwickra.dylib";
            default -> "libwickra.so";
        };
    }
}
