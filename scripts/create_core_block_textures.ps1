Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.Drawing

Add-Type -ReferencedAssemblies "System.Drawing" -TypeDefinition @"
using System;
using System.Drawing;
using System.Drawing.Imaging;
using System.IO;
using System.Runtime.InteropServices;

public static class VoxelVerseTextureGen
{
    const int Size = 512;

    static int Clamp(int v)
    {
        return Math.Max(0, Math.Min(255, v));
    }

    static int Hash2(int x, int y, int seed)
    {
        unchecked
        {
            int n = x * 374761393 + y * 668265263 + seed * 1442695041;
            n = (n ^ (n >> 13)) * 1274126177;
            return (n ^ (n >> 16)) & 255;
        }
    }

    static void WritePng(string path, byte[] bytes)
    {
        Directory.CreateDirectory(Path.GetDirectoryName(path));
        using (var bitmap = new Bitmap(Size, Size, PixelFormat.Format32bppArgb))
        {
            var rect = new Rectangle(0, 0, Size, Size);
            var data = bitmap.LockBits(rect, ImageLockMode.WriteOnly, bitmap.PixelFormat);
            try
            {
                Marshal.Copy(bytes, 0, data.Scan0, bytes.Length);
            }
            finally
            {
                bitmap.UnlockBits(data);
            }
            bitmap.Save(path, ImageFormat.Png);
        }
    }

    public static void SaveMaterial(
        string root,
        string block,
        string name,
        int r,
        int g,
        int b,
        int roughness,
        int normalStrength,
        int seed,
        string kind)
    {
        byte[] albedo = new byte[Size * Size * 4];
        byte[] normal = new byte[Size * Size * 4];
        byte[] rough = new byte[Size * Size * 4];

        for (int y = 0; y < Size; y++)
        {
            for (int x = 0; x < Size; x++)
            {
                int cellX = x / 64;
                int cellY = y / 64;
                int smallX = x / 16;
                int smallY = y / 16;
                int large = Hash2(cellX, cellY, seed) - 128;
                int small = Hash2(smallX, smallY, seed + 7) - 128;
                int grain = Hash2(x, y, seed + 19) - 128;
                int shade = (int)(large * 0.22 + small * 0.10 + grain * 0.018);

                double u = x / (double)(Size - 1);
                double v = y / (double)(Size - 1);
                int highlight = (int)(14.0 * (1.0 - ((u + v) * 0.5)));
                int edge = (x < 10 || y < 10 || x > Size - 11 || y > Size - 11) ? -10 : 0;

                if (kind == "grass_top" && (((x + y * 2) % 37) < 5)) shade += 20;
                if (kind == "grass_side" && y < 190) shade += (int)(30 * (1.0 - (y / 190.0)));
                if (kind == "ice") highlight += (int)(18 * Math.Abs(Math.Sin((x + y) / 47.0)));
                if (kind == "snow") highlight += (int)(10 * Math.Abs(Math.Sin(x / 39.0)));
                if (kind == "gravel" && ((Hash2(x / 28, y / 28, seed + 99) & 7) == 0)) shade -= 22;
                if (kind == "sand" && ((x + y) % 53) < 2) shade += 12;

                int rr = Clamp(r + shade + highlight + edge);
                int gg = Clamp(g + shade + highlight + edge);
                int bb = Clamp(b + shade + highlight + edge);

                int idx = ((y * Size) + x) * 4;
                albedo[idx + 0] = (byte)bb;
                albedo[idx + 1] = (byte)gg;
                albedo[idx + 2] = (byte)rr;
                albedo[idx + 3] = 255;

                double dx = (Hash2(smallX, smallY, seed + 31) - 128) / 128.0;
                double dy = (Hash2(smallX + 3, smallY - 2, seed + 43) - 128) / 128.0;
                int nx = Clamp((int)(128 + dx * normalStrength));
                int ny = Clamp((int)(128 + dy * normalStrength));
                normal[idx + 0] = 238;
                normal[idx + 1] = (byte)ny;
                normal[idx + 2] = (byte)nx;
                normal[idx + 3] = 255;

                int rv = Clamp(roughness + (int)(large * 0.08 + small * 0.05));
                rough[idx + 0] = (byte)rv;
                rough[idx + 1] = (byte)rv;
                rough[idx + 2] = (byte)rv;
                rough[idx + 3] = 255;
            }
        }

        string dir = Path.Combine(root, block);
        WritePng(Path.Combine(dir, name + "_albedo.png"), albedo);
        WritePng(Path.Combine(dir, name + "_normal.png"), normal);
        WritePng(Path.Combine(dir, name + "_roughness.png"), rough);
    }
}
"@

$rootDir = (Resolve-Path (Join-Path $PSScriptRoot "..\packs\core\textures\blocks")).Path
New-Item -ItemType Directory -Force -Path $rootDir | Out-Null

Get-ChildItem -Path $rootDir -Filter "*.png" -File | Remove-Item -Force

[VoxelVerseTextureGen]::SaveMaterial($rootDir, "core", "core_all", 58, 48, 50, 232, 8, 101, "core")
[VoxelVerseTextureGen]::SaveMaterial($rootDir, "dirt", "dirt_all", 144, 88, 42, 220, 16, 211, "dirt")
[VoxelVerseTextureGen]::SaveMaterial($rootDir, "grass", "grass_top", 62, 162, 58, 224, 13, 307, "grass_top")
[VoxelVerseTextureGen]::SaveMaterial($rootDir, "grass", "grass_side", 84, 138, 56, 226, 12, 401, "grass_side")
[VoxelVerseTextureGen]::SaveMaterial($rootDir, "gravel", "gravel_all", 114, 112, 108, 238, 22, 503, "gravel")
[VoxelVerseTextureGen]::SaveMaterial($rootDir, "ice", "ice_all", 176, 226, 246, 94, 7, 607, "ice")
[VoxelVerseTextureGen]::SaveMaterial($rootDir, "sand", "sand_all", 214, 194, 122, 206, 8, 709, "sand")
[VoxelVerseTextureGen]::SaveMaterial($rootDir, "snow", "snow_all", 232, 242, 248, 190, 6, 811, "snow")
[VoxelVerseTextureGen]::SaveMaterial($rootDir, "stone", "stone_all", 122, 122, 128, 230, 14, 919, "stone")

Write-Host "Wrote 512x512 stylized PBR-lite block textures to $rootDir"
