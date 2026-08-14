Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public class ShellHelper {
    [DllImport("shell32.dll", CharSet = CharSet.Auto, SetLastError = true)]
    public static extern void SHChangeNotify(uint wEventId, uint uFlags, IntPtr dwItem1, IntPtr dwItem2);
}
"@

# SHCNE_ASSOCCHANGED = 0x08000000, SHCNF_IDLIST = 0x0000
[ShellHelper]::SHChangeNotify(0x08000000, 0, [IntPtr]::Zero, [IntPtr]::Zero)
Write-Host "Windows Shell icon cache refreshed!"
