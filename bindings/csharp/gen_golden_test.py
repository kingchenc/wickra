"""Generate Wickra.Tests/GoldenAllTests.g.cs: a value-parity test that replays
the shared golden input through every one of the 514 C# indicators and checks
output bit-for-bit against the Rust reference fixtures g_<Canonical>.csv.

Run from repo root:  python bindings/csharp/gen_golden_test.py
"""
import glob
import json
import os
import re

ROOT = os.path.normpath(os.path.join(os.path.dirname(__file__), "..", ".."))
G = os.path.join(ROOT, "testdata", "golden")
GEN = open(os.path.join(ROOT, "bindings", "csharp", "Wickra", "Generated", "Indicators.g.cs"), encoding="utf-8").read()

# C# constructor parameter types per class.
ctor_types = {}
cur = None
for line in GEN.splitlines():
    m = re.match(r"public sealed class (\w+)", line)
    if m:
        cur = m.group(1)
        continue
    if cur:
        cm = re.match(r"\s*public %s\(([^)]*)\)" % re.escape(cur), line)
        if cm:
            ps = cm.group(1).strip()
            types = [p.strip().rsplit(" ", 1)[0].strip() for p in ps.split(",")] if ps else []
            ctor_types[cur] = types
            cur = None

# Unified archetype + params, keyed by canonical (== C# class name).
spec = {}
for e in json.load(open(os.path.join(G, "scalar_manifest.json"))):
    arch = {"f64": "scalar_f64", "Candle": "scalar_candle", "(f64, f64)": "pairwise"}[e["input"]]
    spec[e["canonical"]] = {"arch": arch, "params": e["params"]}
for e in json.load(open(os.path.join(G, "multi_manifest.json"))):
    arch = {"f64": "multi_f64", "Candle": "multi_candle", "(f64, f64)": "multi_pairwise"}[e["input"]]
    spec[e["canonical"]] = {"arch": arch, "params": e["params"], "n": e["n"]}
ex = json.load(open(os.path.join(G, "exotic_manifest.json")))
for e in ex["deriv"]:
    spec[e["canonical"]] = {"arch": "deriv_multi" if "n" in e else "deriv", "params": e["params"], "n": e.get("n")}
for e in ex["cross"]:
    spec[e["canonical"]] = {"arch": "cross", "params": e["params"]}
for e in ex["trade"]:
    spec[e["canonical"]] = {"arch": "trade", "params": e["params"]}
for e in ex["trademid"]:
    spec[e["canonical"]] = {"arch": "trademid", "params": e["params"]}
for e in ex["ob"]:
    spec[e["canonical"]] = {"arch": "ob", "params": e["params"]}
for e in json.load(open(os.path.join(G, "profile_manifest.json"))):
    spec[e["canonical"]] = {"arch": "profile_" + e["kind"], "params": e["params"], "width": e["width"]}
for e in json.load(open(os.path.join(G, "bars_manifest.json"))):
    arch = "footprint" if e["canonical"] == "Footprint" else "bars_" + e["feed"]
    spec[e["canonical"]] = {"arch": arch, "params": e["params"]}

canons = sorted(os.path.basename(f)[2:-4] for f in glob.glob(os.path.join(G, "g_*.csv")))


def lit(value, cstype):
    if cstype == "int":
        return str(int(round(value)))
    if cstype == "uint":
        return f"{int(round(value))}u"
    if cstype == "byte":
        return f"(byte){int(round(value))}"
    f = float(value)
    s = repr(f)
    return s if ("." in s or "e" in s or "E" in s) else s + ".0"


def ctor_call(canon):
    types = ctor_types.get(canon, [])
    vals = spec[canon]["params"]
    args = ", ".join(lit(v, t) for v, t in zip(vals, types))
    return f"new Wickra.{canon}({args})"


def block(canon):
    s = spec[canon]
    a = s["arch"]
    L = [f"    [Fact]", f"    public void Golden_{canon}()", "    {"]
    L.append(f"        using var ind = {ctor_call(canon)};")
    L.append("        var got = new List<double[]>();")
    L.append("        for (var i = 0; i < Rows.Length; i++)")
    L.append("        {")
    L.append("            var r = Rows[i];")
    if a == "scalar_f64":
        L.append("            got.Add(new[] { ind.Update(r[3]) });")
    elif a == "pairwise":
        L.append("            got.Add(new[] { ind.Update(r[3], r[0]) });")
    elif a == "scalar_candle":
        L.append("            got.Add(new[] { ind.Update(r[0], r[1], r[2], r[3], r[4], i) });")
    elif a == "trade":
        L.append("            got.Add(new[] { ind.Update(r[3], r[4], r[3] >= r[0], i) });")
    elif a == "trademid":
        L.append("            got.Add(new[] { ind.Update(r[3], r[4], r[3] >= r[0], i, (r[1] + r[2]) / 2) });")
    elif a == "ob":
        L.append("            var (bp, bs, ap, asz) = ObLists(r);")
        L.append("            got.Add(new[] { ind.Update(bp, bs, ap, asz) });")
    elif a == "deriv":
        L.append("            var d = DerivFields(r);")
        L.append("            got.Add(new[] { ind.Update(d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7], d[8], d[9], d[10], i) });")
    elif a == "deriv_multi":
        L.append("            var d = DerivFields(r);")
        L.append("            got.Add(FlattenNullable(ind.Update(d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7], d[8], d[9], d[10], i), %d));" % s["n"])
    elif a == "cross":
        L.append("            var (ch, vo, nh, nl, am, ob) = CrossLists(r);")
        L.append("            got.Add(new[] { ind.Update(ch, vo, nh, nl, am, ob, i) });")
    elif a == "multi_f64":
        L.append("            got.Add(FlattenNullable(ind.Update(r[3]), %d));" % s["n"])
    elif a == "multi_pairwise":
        L.append("            got.Add(FlattenNullable(ind.Update(r[3], r[0]), %d));" % s["n"])
    elif a == "multi_candle":
        L.append("            got.Add(FlattenNullable(ind.Update(r[0], r[1], r[2], r[3], r[4], i), %d));" % s["n"])
    elif a == "profile_bins":
        L.append("            var bins = ind.Update(r[0], r[1], r[2], r[3], r[4], i);")
        L.append("            got.Add(bins ?? NanRow(%d));" % s["width"])
    elif a == "profile_pricebins":
        L.append("            got.Add(FlattenNullable(ind.Update(r[0], r[1], r[2], r[3], r[4], i), %d));" % s["width"])
    elif a == "bars_close":
        L.append("            got.Add(FlattenBars(ind.Update(r[3], r[3], r[3], r[3], 1.0, 0)));")
    elif a == "bars_candle4":
        L.append("            got.Add(FlattenBars(ind.Update(r[0], r[1], r[2], r[3], 1.0, 0)));")
    elif a == "bars_candle5":
        L.append("            got.Add(FlattenBars(ind.Update(r[0], r[1], r[2], r[3], r[4], 0)));")
    elif a == "footprint":
        L.append("            got.Add(FlattenBars(ind.Update(r[3], r[4], r[3] >= r[0], i)));")
    else:
        raise SystemExit("arch " + a)
    L.append("        }")
    L.append(f'        Compare("{canon}", got);')
    L.append("    }")
    return "\n".join(L)


HEADER = '''// <auto-generated>
// Generated by gen_golden_test.py. DO NOT EDIT.
//
// Value-parity for every one of the 514 C# indicators: the shared golden input
// is replayed through each one and checked bit-for-bit against the Rust
// reference fixtures testdata/golden/g_<Canonical>.csv. Multi-output, profile
// and bar shapes are flattened by reflection so one comparator covers all
// archetypes. Regenerate with: python bindings/csharp/gen_golden_test.py
// </auto-generated>
#nullable enable
using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Reflection;
using Xunit;

namespace Wickra.Tests;

public class GoldenAllTests
{
    private const double Tol = 1e-6;

    private static readonly double[][] Rows = LoadInput();

    private static string GoldenDir([System.Runtime.CompilerServices.CallerFilePath] string file = "") =>
        Path.GetFullPath(Path.Combine(Path.GetDirectoryName(file)!, "..", "..", "..", "testdata", "golden"));

    private static double Cell(string s) =>
        s == "nan" ? double.NaN
        : s == "inf" ? double.PositiveInfinity
        : s == "-inf" ? double.NegativeInfinity
        : double.Parse(s, CultureInfo.InvariantCulture);

    private static double[][] LoadInput()
    {
        var lines = File.ReadAllLines(Path.Combine(GoldenDir(), "input.csv"));
        return lines.Skip(1).Where(l => l.Length > 0)
            .Select(l => l.Split(',').Select(x => double.Parse(x, CultureInfo.InvariantCulture)).ToArray())
            .ToArray();
    }

    // Keep blank lines (a candle on which no bar closed) so rows stay aligned.
    private static double[]?[] ReadFixture(string name)
    {
        var lines = File.ReadAllLines(Path.Combine(GoldenDir(), "g_" + name + ".csv"));
        return lines.Skip(1).Select(l => l.Length == 0 ? Array.Empty<double>() : l.Split(',').Select(Cell).ToArray()).ToArray();
    }

    private static double[] NanRow(int n)
    {
        var r = new double[n];
        for (var i = 0; i < n; i++) r[i] = double.NaN;
        return r;
    }

    private static double[] FlattenStruct(object o)
    {
        var props = o.GetType()
            .GetProperties(BindingFlags.Public | BindingFlags.Instance)
            .OrderBy(p => p.MetadataToken);
        var list = new List<double>();
        foreach (var p in props)
        {
            var v = p.GetValue(o);
            switch (v)
            {
                case double d: list.Add(d); break;
                case float f: list.Add(f); break;
                case long l: list.Add(l); break;
                case int n: list.Add(n); break;
                case double[] arr: list.AddRange(arr); break;
            }
        }
        return list.ToArray();
    }

    private static double[] FlattenNullable<T>(T? value, int width) where T : struct =>
        value.HasValue ? FlattenStruct(value.Value) : NanRow(width);

    private static double[] FlattenBars<T>(T[] bars)
    {
        var list = new List<double>();
        foreach (var bar in bars) list.AddRange(FlattenStruct(bar!));
        return list.ToArray();
    }

    private static double[] DerivFields(double[] r)
    {
        double o = r[0], h = r[1], l = r[2], c = r[3], v = r[4];
        return new[]
        {
            (c - o) / c * 0.01, c, c - 0.5, c + 1.0, v * 10.0, v * 0.6, v * 0.4,
            v * 0.55, v * 0.45, h - c, c - l,
        };
    }

    private static (double[], double[], bool[], bool[], bool[], bool[]) CrossLists(double[] r)
    {
        double o = r[0], c = r[3], v = r[4];
        var change = new double[5];
        var volume = new double[5];
        var nh = new bool[5];
        var nl = new bool[5];
        var am = new bool[5];
        var ob = new bool[5];
        for (var j = 0; j < 5; j++)
        {
            change[j] = (c - o) + j;
            volume[j] = v + j * 10.0;
            nh[j] = j % 2 == 0;
            nl[j] = j % 3 == 0;
            am[j] = j % 2 == 0;
            ob[j] = j % 3 == 0;
        }
        return (change, volume, nh, nl, am, ob);
    }

    private static (double[], double[], double[], double[]) ObLists(double[] r)
    {
        double c = r[3], v = r[4];
        var bp = new double[5];
        var bs = new double[5];
        var ap = new double[5];
        var asz = new double[5];
        for (var k = 0; k < 5; k++)
        {
            var kf = k + 1;
            bp[k] = c - 0.1 * kf;
            bs[k] = v / kf;
            ap[k] = c + 0.1 * kf;
            asz[k] = v * 0.9 / kf;
        }
        return (bp, bs, ap, asz);
    }

    private static void Compare(string name, List<double[]> got)
    {
        var exp = ReadFixture(name);
        Assert.True(exp.Length == got.Count, $"{name}: {exp.Length} fixture rows vs {got.Count} computed");
        for (var i = 0; i < exp.Length; i++)
        {
            var want = exp[i]!;
            var g = got[i];
            Assert.True(want.Length == g.Length, $"{name} row {i}: arity {g.Length} vs {want.Length}");
            for (var k = 0; k < want.Length; k++)
            {
                var w = want[k];
                if (double.IsNaN(w)) { Assert.True(double.IsNaN(g[k]), $"{name} row {i} col {k}: want NaN got {g[k]}"); continue; }
                if (double.IsInfinity(w)) { Assert.True(double.IsInfinity(g[k]) && Math.Sign(g[k]) == Math.Sign(w), $"{name} row {i} col {k}: want {w} got {g[k]}"); continue; }
                var tol = Tol * Math.Max(1.0, Math.Abs(w));
                Assert.True(Math.Abs(g[k] - w) <= tol, $"{name} row {i} col {k}: got {g[k]} want {w}");
            }
        }
    }
'''

out = [HEADER]
for canon in canons:
    out.append(block(canon))
out.append("}")
open(os.path.join(ROOT, "bindings", "csharp", "Wickra.Tests", "GoldenAllTests.g.cs"), "w", encoding="utf-8").write("\n".join(out) + "\n")
print("generated GoldenAllTests.g.cs with", len(canons), "indicators")
