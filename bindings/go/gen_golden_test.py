"""Generate golden_all_test.go: a value-parity test that replays the shared
golden input through every one of the 514 Go indicators and checks output
bit-for-bit against the Rust-generated g_<Canonical>.csv fixtures.

Run from repo root:  python bindings/go/gen_golden_test.py
"""
import glob
import json
import os
import re

ROOT = os.path.normpath(os.path.join(os.path.dirname(__file__), "..", ".."))
G = os.path.join(ROOT, "testdata", "golden")
GEN = open(os.path.join(ROOT, "bindings", "go", "indicators_gen.go"), encoding="utf-8").read()

# Canonical core Indicator::name() per indicator, shared across every binding.
NAMES = json.load(open(os.path.join(G, "names.json")))

# Go constructor parameter types, keyed by canonical (== Go type name).
ctor_types = {}
for m in re.finditer(r"func New(\w+)\(([^)]*)\)\s*\(\*\w+, error\)", GEN):
    name, ps = m.group(1), m.group(2).strip()
    types = []
    if ps:
        for p in ps.split(","):
            p = p.strip()
            _, _, ty = p.partition(" ")
            types.append(ty.strip())
    ctor_types[name] = types

# Unified archetype + params, keyed by canonical.
spec = {}  # canon -> dict(arch, params, width?, n?)
scal = json.load(open(os.path.join(G, "scalar_manifest.json")))
for e in scal:
    inp = e["input"]
    arch = {"f64": "scalar_f64", "Candle": "scalar_candle", "(f64, f64)": "pairwise"}[inp]
    spec[e["canonical"]] = {"arch": arch, "params": e["params"]}
for e in json.load(open(os.path.join(G, "multi_manifest.json"))):
    inp = e["input"]
    arch = {"f64": "multi_f64", "Candle": "multi_candle", "(f64, f64)": "multi_pairwise"}[inp]
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


def go_param(value, gotype):
    intlike = gotype in ("int", "int32", "int64", "uint", "uintptr", "usize")
    if intlike:
        return str(int(round(value)))
    # float64
    return repr(float(value)) if "." in repr(float(value)) or "e" in repr(float(value)) else f"{float(value)}"


def ctor_call(canon):
    types = ctor_types.get(canon, [])
    vals = spec[canon]["params"]
    args = ", ".join(go_param(v, t) for v, t in zip(vals, types))
    return f"New{canon}({args})"


# Update-call expression + output handling per archetype.
def block(canon):
    s = spec[canon]
    a = s["arch"]
    ctor = ctor_call(canon)
    lines = [f'\tt.Run("{canon}", func(t *testing.T) {{']
    lines.append(f"\t\tind, err := {ctor}")
    lines.append('\t\tif err != nil {')
    lines.append(f'\t\t\tt.Fatalf("new {canon}: %v", err)')
    lines.append("\t\t}")
    lines.append(f'\t\tif n := ind.Name(); n != {json.dumps(NAMES[canon])} {{')
    lines.append(f'\t\t\tt.Errorf("name: got %q want %q", n, {json.dumps(NAMES[canon])})')
    lines.append("\t\t}")
    lines.append("\t\tgot := make([][]float64, len(rows))")
    lines.append("\t\tfor i, r := range rows {")
    if a == "scalar_f64":
        upd = "ind.Update(r[3])"
        lines.append(f"\t\t\tgot[i] = []float64{{{upd}}}")
    elif a == "pairwise":
        lines.append("\t\t\tgot[i] = []float64{ind.Update(r[3], r[0])}")
    elif a == "scalar_candle":
        lines.append("\t\t\tgot[i] = []float64{ind.Update(r[0], r[1], r[2], r[3], r[4], int64(i))}")
    elif a == "trade":
        lines.append("\t\t\tgot[i] = []float64{ind.Update(r[3], r[4], r[3] >= r[0], int64(i))}")
    elif a == "trademid":
        lines.append("\t\t\tgot[i] = []float64{ind.Update(r[3], r[4], r[3] >= r[0], int64(i), (r[1]+r[2])/2)}")
    elif a == "ob":
        lines.append("\t\t\tbp, bs, ap, as_ := obLists(r)")
        lines.append("\t\t\tgot[i] = []float64{ind.Update(bp, bs, ap, as_)}")
    elif a == "deriv":
        lines.append("\t\t\td := derivFields(r)")
        lines.append("\t\t\tgot[i] = []float64{ind.Update(d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7], d[8], d[9], d[10], int64(i))}")
    elif a == "deriv_multi":
        lines.append("\t\t\td := derivFields(r)")
        lines.append("\t\t\tout, ok := ind.Update(d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7], d[8], d[9], d[10], int64(i))")
        lines.append(f"\t\t\tgot[i] = reflectRow(out, ok, {s['n']})")
    elif a == "cross":
        lines.append("\t\t\tch, vo, nh, nl, am, ob_ := crossLists(r)")
        lines.append("\t\t\tgot[i] = []float64{ind.Update(ch, vo, nh, nl, am, ob_, int64(i))}")
    elif a in ("multi_f64",):
        lines.append("\t\t\tout, ok := ind.Update(r[3])")
        lines.append(f"\t\t\tgot[i] = reflectRow(out, ok, {s['n']})")
    elif a == "multi_pairwise":
        lines.append("\t\t\tout, ok := ind.Update(r[3], r[0])")
        lines.append(f"\t\t\tgot[i] = reflectRow(out, ok, {s['n']})")
    elif a == "multi_candle":
        lines.append("\t\t\tout, ok := ind.Update(r[0], r[1], r[2], r[3], r[4], int64(i))")
        lines.append(f"\t\t\tgot[i] = reflectRow(out, ok, {s['n']})")
    elif a == "profile_bins":
        lines.append("\t\t\tbins, ok := ind.Update(r[0], r[1], r[2], r[3], r[4], int64(i))")
        lines.append(f"\t\t\tif ok {{ got[i] = bins }} else {{ got[i] = nanRow({s['width']}) }}")
    elif a == "profile_pricebins":
        lines.append("\t\t\tout, ok := ind.Update(r[0], r[1], r[2], r[3], r[4], int64(i))")
        lines.append(f"\t\t\tgot[i] = reflectRow(out, ok, {s['width']})")
    elif a == "bars_close":
        lines.append("\t\t\tgot[i] = flattenBars(ind.Update(r[3], r[3], r[3], r[3], 1.0, 0))")
    elif a == "bars_candle4":
        lines.append("\t\t\tgot[i] = flattenBars(ind.Update(r[0], r[1], r[2], r[3], 1.0, 0))")
    elif a == "bars_candle5":
        lines.append("\t\t\tgot[i] = flattenBars(ind.Update(r[0], r[1], r[2], r[3], r[4], 0))")
    elif a == "footprint":
        lines.append("\t\t\tgot[i] = flattenBars(ind.Update(r[3], r[4], r[3] >= r[0], int64(i)))")
    else:
        raise SystemExit("unknown arch " + a)
    lines.append("\t\t}")
    lines.append(f'\t\tcompareGolden(t, "{canon}", got)')
    lines.append("\t})")
    return "\n".join(lines)


HEADER = '''// Code generated by gen_golden_test.py. DO NOT EDIT.
//
// Value-parity for every one of the 514 Go indicators: the shared golden input
// is replayed through each one and checked bit-for-bit against the Rust
// reference fixtures testdata/golden/g_<Canonical>.csv. Multi-output, profile
// and bar shapes are flattened by reflection so a single comparator covers all
// archetypes. Regenerate with: python bindings/go/gen_golden_test.py

package wickra

import (
\t"bufio"
\t"math"
\t"os"
\t"reflect"
\t"strings"
\t"testing"
)

// readGoldenRaw keeps blank lines (a candle on which no bar closed) so bar rows
// stay aligned to the input; non-bar fixtures contain no blank lines.
func readGoldenRaw(t *testing.T, name string) [][]string {
\tt.Helper()
\tf, err := os.Open("../../testdata/golden/" + name + ".csv")
\tif err != nil {
\t\tt.Fatalf("open %s: %v", name, err)
\t}
\tdefer f.Close()
\tvar rows [][]string
\tsc := bufio.NewScanner(f)
\tsc.Buffer(make([]byte, 0, 1024*1024), 1024*1024)
\tfirst := true
\tfor sc.Scan() {
\t\tline := sc.Text()
\t\tif first {
\t\t\tfirst = false
\t\t\tcontinue
\t\t}
\t\tif line == "" {
\t\t\trows = append(rows, []string{})
\t\t\tcontinue
\t\t}
\t\trows = append(rows, strings.Split(line, ","))
\t}
\treturn rows
}

func nanRow(n int) []float64 {
\tr := make([]float64, n)
\tfor i := range r {
\t\tr[i] = math.NaN()
\t}
\treturn r
}

func reflectRow(out any, ok bool, width int) []float64 {
\tif !ok {
\t\treturn nanRow(width)
\t}
\tv := reflect.ValueOf(out)
\trow := make([]float64, 0, width)
\tfor k := 0; k < v.NumField(); k++ {
\t\trow = appendField(row, v.Field(k))
\t}
\treturn row
}

func appendField(row []float64, f reflect.Value) []float64 {
\tswitch f.Kind() {
\tcase reflect.Float64, reflect.Float32:
\t\treturn append(row, f.Float())
\tcase reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
\t\treturn append(row, float64(f.Int()))
\tcase reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64, reflect.Uintptr:
\t\treturn append(row, float64(f.Uint()))
\tcase reflect.Slice:
\t\tfor j := 0; j < f.Len(); j++ {
\t\t\trow = appendField(row, f.Index(j))
\t\t}
\t\treturn row
\tdefault:
\t\treturn row
\t}
}

func flattenBars(bars any) []float64 {
\tv := reflect.ValueOf(bars)
\trow := []float64{}
\tfor i := 0; i < v.Len(); i++ {
\t\tbar := v.Index(i)
\t\tfor k := 0; k < bar.NumField(); k++ {
\t\t\trow = appendField(row, bar.Field(k))
\t\t}
\t}
\treturn row
}

// Synthetic feeds derived from one OHLCV row, identical to gen_golden's Rust
// construction (DerivativesTick / CrossSection / OrderBook).
func derivFields(r []float64) [11]float64 {
\to, h, l, c, v := r[0], r[1], r[2], r[3], r[4]
\treturn [11]float64{
\t\t(c - o) / c * 0.01, // funding_rate
\t\tc,                  // mark_price
\t\tc - 0.5,            // index_price
\t\tc + 1.0,            // futures_price
\t\tv * 10.0,           // open_interest
\t\tv * 0.6,            // long_size
\t\tv * 0.4,            // short_size
\t\tv * 0.55,           // taker_buy_volume
\t\tv * 0.45,           // taker_sell_volume
\t\th - c,              // long_liquidation
\t\tc - l,              // short_liquidation
\t}
}

func crossLists(r []float64) ([]float64, []float64, []bool, []bool, []bool, []bool) {
\to, c, v := r[0], r[3], r[4]
\tchange := make([]float64, 5)
\tvolume := make([]float64, 5)
\tnewHigh := make([]bool, 5)
\tnewLow := make([]bool, 5)
\taboveMa := make([]bool, 5)
\tonBuy := make([]bool, 5)
\tfor j := 0; j < 5; j++ {
\t\tjf := float64(j)
\t\tchange[j] = (c - o) + jf
\t\tvolume[j] = v + jf*10.0
\t\tnewHigh[j] = j%2 == 0
\t\tnewLow[j] = j%3 == 0
\t\taboveMa[j] = j%2 == 0
\t\tonBuy[j] = j%3 == 0
\t}
\treturn change, volume, newHigh, newLow, aboveMa, onBuy
}

func obLists(r []float64) ([]float64, []float64, []float64, []float64) {
\tc, v := r[3], r[4]
\tbidPx := make([]float64, 5)
\tbidSz := make([]float64, 5)
\taskPx := make([]float64, 5)
\taskSz := make([]float64, 5)
\tfor k := 0; k < 5; k++ {
\t\tkf := float64(k + 1)
\t\tbidPx[k] = c - 0.1*kf
\t\tbidSz[k] = v / kf
\t\taskPx[k] = c + 0.1*kf
\t\taskSz[k] = v * 0.9 / kf
\t}
\treturn bidPx, bidSz, askPx, askSz
}

func compareGolden(t *testing.T, name string, got [][]float64) {
\tt.Helper()
\texp := readGoldenRaw(t, "g_"+name)
\tif len(exp) != len(got) {
\t\tt.Fatalf("%s: %d fixture rows vs %d computed", name, len(exp), len(got))
\t}
\tfor i := range exp {
\t\tif len(exp[i]) != len(got[i]) {
\t\t\tt.Fatalf("%s row %d: arity %d vs %d", name, i, len(got[i]), len(exp[i]))
\t\t}
\t\tfor k := range exp[i] {
\t\t\twant := goldenCell(exp[i][k])
\t\t\tg := got[i][k]
\t\t\tif math.IsNaN(want) {
\t\t\t\tif !math.IsNaN(g) {
\t\t\t\t\tt.Fatalf("%s row %d col %d: want NaN got %v", name, i, k, g)
\t\t\t\t}
\t\t\t\tcontinue
\t\t\t}
\t\t\tif math.IsInf(want, 0) {
\t\t\t\tif !math.IsInf(g, 0) || (g > 0) != (want > 0) {
\t\t\t\t\tt.Fatalf("%s row %d col %d: want %v got %v", name, i, k, want, g)
\t\t\t\t}
\t\t\t\tcontinue
\t\t\t}
\t\t\ttol := goldenTol * math.Max(1.0, math.Abs(want))
\t\t\tif math.Abs(g-want) > tol {
\t\t\t\tt.Fatalf("%s row %d col %d: got %v want %v", name, i, k, g, want)
\t\t\t}
\t\t}
\t}
}

func TestGoldenAll(t *testing.T) {
\trows := goldenInput(t)
'''

# bars need blank-line-preserving fixture reads; reuse readGolden but it skips
# blanks. We need a raw reader for bars and input.
out = [HEADER]
for canon in canons:
    out.append(block(canon))
out.append("}")
open(os.path.join(ROOT, "bindings", "go", "golden_all_test.go"), "w", encoding="utf-8").write("\n".join(out) + "\n")
print("generated golden_all_test.go with", len(canons), "indicators")
