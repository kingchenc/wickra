using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

[assembly: InternalsVisibleTo("Wickra.Tests")]

namespace Wickra;

/// <summary>
/// Native library resolution for the Wickra C ABI.
/// </summary>
/// <remarks>
/// <para>
/// When consumed as a NuGet package the native library ships under
/// <c>runtimes/&lt;rid&gt;/native/</c>. On Windows it is named
/// <c>wickra_native.dll</c> rather than <c>wickra.dll</c>: the managed assembly
/// is <c>Wickra.dll</c>, and Windows file names are case-insensitive, so the two
/// occupy the same relative path once a RID-specific publish flattens
/// <c>runtimes/</c> into the output folder. The SDK rejects that outright with
/// <c>NETSDK1152</c>, which made <c>dotnet publish</c> fail for every Windows
/// application referencing the package.
/// </para>
/// <para>
/// For local development (project reference against a cargo build) the resolver
/// additionally walks up the directory tree to locate <c>target/release</c> or
/// <c>target/debug</c>. Every candidate is validated to actually export the
/// Wickra ABI before it is accepted, so an unrelated library of the same name
/// cannot shadow the real one, and resolution never falls back to the runtime's
/// own unvalidated probing.
/// </para>
/// </remarks>
internal static class WickraNative
{
    /// <summary>The library name passed to <c>[LibraryImport]</c>.</summary>
    internal const string LibraryName = "wickra";

    /// <summary>
    /// The name the native library carries inside the NuGet package. It differs
    /// from <see cref="LibraryName"/> so that it cannot collide with the managed
    /// <c>Wickra.dll</c> on a case-insensitive file system.
    /// </summary>
    internal const string PackagedLibraryName = "wickra_native";

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

    internal static nint Resolve(string libraryName, System.Reflection.Assembly assembly, DllImportSearchPath? searchPath)
    {
        if (libraryName != LibraryName)
        {
            return nint.Zero;
        }

        var probed = new List<string>();

        // 1. The packaged name, then the plain one: NuGet's runtimes/ layout and
        //    app-local copies. Accept only a library that genuinely exports the
        //    Wickra ABI; otherwise discard it and keep looking.
        foreach (var name in new[] { PackagedLibraryName, LibraryName })
        {
            probed.Add(name);
            if (NativeLibrary.TryLoad(name, assembly, searchPath, out var handle))
            {
                if (Exports(handle))
                {
                    return handle;
                }

                NativeLibrary.Free(handle);
            }
        }

        // 2. Development fallback: locate the cargo build output.
        var fileName = NativeFileName();
        var dir = AppContext.BaseDirectory;
        for (var i = 0; i < 16 && dir is not null; i++)
        {
            foreach (var profile in new[] { "release", "debug" })
            {
                var candidate = Path.Combine(dir, "target", profile, fileName);
                probed.Add(candidate);
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

        // Returning zero here would hand the load back to the runtime's default
        // probing, which does none of the validation above -- so a same-named
        // library that exports nothing would be accepted and the caller would see
        // an EntryPointNotFoundException from deep inside an unrelated call.
        throw new DllNotFoundException(
            $"Wickra could not load the native library '{fileName}'. Install the Wickra NuGet package for "
            + $"this platform, or build the C ABI with `cargo build -p wickra-c --release`. Probed: "
            + string.Join(", ", probed));
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
