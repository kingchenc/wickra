using System.Reflection;
using System.Runtime.InteropServices;
using Xunit;

namespace Wickra.Tests;

/// <summary>
/// The native library ships inside the NuGet package under
/// <c>runtimes/&lt;rid&gt;/native/</c>. Naming it <c>wickra.dll</c> gave it the
/// same relative path as the managed <c>Wickra.dll</c> once a RID-specific
/// publish flattens <c>runtimes/</c>, because Windows file names are
/// case-insensitive — and the SDK rejects that with <c>NETSDK1152</c>, so
/// <c>dotnet publish</c> failed outright for every Windows application that
/// referenced the package.
/// </summary>
public class NativeResolutionTests
{
    private static string ManagedAssemblyFileName =>
        Path.GetFileName(typeof(Sma).Assembly.Location);

    [Fact]
    public void PackagedNativeNameCannotCollideWithTheManagedAssembly()
    {
        var packaged = WickraNative.PackagedLibraryName + ".dll";
        Assert.False(
            string.Equals(packaged, ManagedAssemblyFileName, StringComparison.OrdinalIgnoreCase),
            $"the packaged native library '{packaged}' shares a path with the managed assembly");
    }

    [Fact]
    public void ThePlainNameIsTheOneThatWouldCollide()
    {
        // Pins the reason the packaged name exists: if this ever stops being
        // true the rename can go, and if the rename is reverted the first test
        // fails.
        Assert.Equal(
            WickraNative.LibraryName + ".dll",
            ManagedAssemblyFileName,
            ignoreCase: true);
    }

    [Fact]
    public void TheResolverClaimsOnlyItsOwnLibrary()
    {
        var resolved = WickraNative.Resolve(
            "some-other-library",
            typeof(Sma).Assembly,
            DllImportSearchPath.AssemblyDirectory);

        Assert.Equal(nint.Zero, resolved);
    }

    [Fact]
    public void TheResolverFindsAWorkingLibrary()
    {
        var handle = WickraNative.Resolve(
            WickraNative.LibraryName,
            typeof(Sma).Assembly,
            DllImportSearchPath.AssemblyDirectory);

        Assert.NotEqual(nint.Zero, handle);
        Assert.True(NativeLibrary.TryGetExport(handle, "wickra_sma_new", out _));
    }
}
