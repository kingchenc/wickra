using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Wickra;

/// <summary>
/// Native library resolution for the Wickra C ABI.
/// </summary>
/// <remarks>
/// When consumed as a NuGet package the native library ships under
/// <c>runtimes/&lt;rid&gt;/native/</c> and the default runtime resolver finds it
/// automatically. For local development (project reference against a cargo build)
/// the resolver additionally walks up the directory tree to locate
/// <c>target/release</c> or <c>target/debug</c>. Every candidate is validated to
/// actually export the Wickra ABI before it is accepted, so an unrelated library
/// of the same name cannot shadow the real one.
/// </remarks>
internal static class WickraNative
{
    /// <summary>The library name passed to <c>[LibraryImport]</c>.</summary>
    internal const string LibraryName = "wickra";

    // Any exported symbol works as a fingerprint; sma_new exists in every build.
    private const string SentinelSymbol = "wickra_sma_new";

    // CA2255 warns against [ModuleInitializer] in libraries, but registering the
    // native-library resolver before any P/Invoke runs is exactly the advanced
    // scenario the attribute exists for: a static constructor would run too late
    // (only on first access to this type), letting the default resolver fail first.
    [System.Diagnostics.CodeAnalysis.SuppressMessage(
        "Usage", "CA2255:The 'ModuleInitializer' attribute should not be used in libraries",
        Justification = "The DllImport resolver must be registered before the first P/Invoke; a static constructor would run too late.")]
    [ModuleInitializer]
    internal static void Register()
    {
        NativeLibrary.SetDllImportResolver(typeof(WickraNative).Assembly, Resolve);
    }

    private static nint Resolve(string libraryName, System.Reflection.Assembly assembly, DllImportSearchPath? searchPath)
    {
        if (libraryName != LibraryName)
        {
            return nint.Zero;
        }

        // 1. Default resolution (NuGet runtimes/ layout, app-local copies). Accept
        //    only if it is genuinely the Wickra ABI; otherwise discard and fall through.
        if (NativeLibrary.TryLoad(libraryName, assembly, searchPath, out var handle))
        {
            if (Exports(handle))
            {
                return handle;
            }

            NativeLibrary.Free(handle);
        }

        // 2. Development fallback: locate the cargo build output.
        var fileName = NativeFileName();
        var dir = AppContext.BaseDirectory;
        for (var i = 0; i < 16 && dir is not null; i++)
        {
            foreach (var profile in new[] { "release", "debug" })
            {
                var candidate = Path.Combine(dir, "target", profile, fileName);
                if (File.Exists(candidate) && NativeLibrary.TryLoad(candidate, out var devHandle))
                {
                    if (Exports(devHandle))
                    {
                        return devHandle;
                    }

                    NativeLibrary.Free(devHandle);
                }
            }

            dir = Path.GetDirectoryName(dir.TrimEnd(Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar));
        }

        return nint.Zero;
    }

    private static bool Exports(nint handle) => NativeLibrary.TryGetExport(handle, SentinelSymbol, out _);

    private static string NativeFileName()
    {
        if (OperatingSystem.IsWindows())
        {
            return "wickra.dll";
        }

        return OperatingSystem.IsMacOS() ? "libwickra.dylib" : "libwickra.so";
    }
}
