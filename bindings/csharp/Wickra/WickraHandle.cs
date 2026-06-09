using Microsoft.Win32.SafeHandles;

namespace Wickra;

/// <summary>
/// Owns an opaque native indicator handle and releases it via the indicator's
/// <c>_free</c> function. One generic handle type backs every indicator; the
/// correct free routine is captured at construction time.
/// </summary>
internal sealed class WickraHandle : SafeHandleZeroOrMinusOneIsInvalid
{
    private readonly Action<nint> _free;

    internal WickraHandle(nint handle, Action<nint> free)
        : base(ownsHandle: true)
    {
        _free = free;
        SetHandle(handle);
    }

    protected override bool ReleaseHandle()
    {
        _free(handle);
        return true;
    }
}
