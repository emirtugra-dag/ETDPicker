Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
using System.Collections.Generic;

public class ResourceInspector {
    public delegate bool EnumResNameProc(IntPtr hModule, IntPtr lpszType, IntPtr lpszName, IntPtr lParam);

    [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Auto)]
    public static extern IntPtr LoadLibraryEx(string lpFileName, IntPtr hReservedNull, uint dwFlags);

    [DllImport("kernel32.dll", SetLastError = true)]
    public static extern bool FreeLibrary(IntPtr hModule);

    [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Auto)]
    public static extern bool EnumResourceNames(IntPtr hModule, IntPtr lpszType, EnumResNameProc lpEnumFunc, IntPtr lParam);

    public const uint LOAD_LIBRARY_AS_DATAFILE = 0x00000002;
    public const uint LOAD_LIBRARY_AS_IMAGE_RESOURCE = 0x00000020;

    public static List<string> ListIcons(string path) {
        var results = new List<string>();
        IntPtr hModule = LoadLibraryEx(path, IntPtr.Zero, LOAD_LIBRARY_AS_DATAFILE | LOAD_LIBRARY_AS_IMAGE_RESOURCE);
        if (hModule == IntPtr.Zero) {
            results.Add("Failed to load library: " + Marshal.GetLastWin32Error());
            return results;
        }

        // RT_GROUP_ICON = 14, RT_ICON = 3
        EnumResourceNames(hModule, (IntPtr)14, (h, type, name, param) => {
            results.Add("GROUP_ICON: " + name.ToString());
            return true;
        }, IntPtr.Zero);

        EnumResourceNames(hModule, (IntPtr)3, (h, type, name, param) => {
            results.Add("ICON: " + name.ToString());
            return true;
        }, IntPtr.Zero);

        // RT_MANIFEST = 24
        EnumResourceNames(hModule, (IntPtr)24, (h, type, name, param) => {
            results.Add("MANIFEST: " + name.ToString());
            return true;
        }, IntPtr.Zero);

        FreeLibrary(hModule);
        return results;
    }
}
"@

Write-Host "=== Inspecting ETDPicker_Portable.exe ==="
$r1 = [ResourceInspector]::ListIcons("C:\Users\vboxuser\Desktop\ETDPicker\dist\ETDPicker_Portable.exe")
$r1 | ForEach-Object { Write-Host $_ }

Write-Host "`n=== Inspecting ETDPicker_Setup.exe ==="
$r2 = [ResourceInspector]::ListIcons("C:\Users\vboxuser\Desktop\ETDPicker\dist\ETDPicker_Setup.exe")
$r2 | ForEach-Object { Write-Host $_ }
