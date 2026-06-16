"""Strategy example: Bollinger-Squeeze breakout with ATR-based stop.

Enters long when the Bollinger Bandwidth has just printed a fresh
6-month low (the squeeze) and price closes above the upper band (the
release). Exits when price closes below entry minus 2 * ATR(14), or
when the upper band rolls back under the entry price. 0.1% fees per
trade.

Educational example. NOT a live trading recommendation.

Run with::

    python -m examples.python.strategy_bollinger_squeeze

Uses the checked-in ``examples/data/btcusdt-1d.csv`` dataset because
daily bars give an interpretable 6-month-low lookback (~180 bars).
"""

from __future__ import annotations

import math
from collections import deque
from pathlib import Path

import wickra as ta

FEE = 0.001
BB_PERIOD = 20
BB_K = 2.0
ATR_PERIOD = 14
ATR_STOP_MULT = 2.0
SQUEEZE_LOOKBACK = 180


def load_candles(path: Path) -> list[dict[str, float]]:
    # Native CandleReader: validates the header, tolerates a UTF-8 BOM and field
    # whitespace, and raises ValueError on a malformed row. No third-party CSV.
    candles = ta.CandleReader(path.read_text(encoding="utf-8")).read()
    # CandleReader yields (open, high, low, close, volume, timestamp) tuples.
    return [
        {"open": o, "high": h, "low": l, "close": c, "volume": v}
        for o, h, l, c, v, _ts in candles
    ]


def print_summary(
    name: str,
    first_price: float,
    last_price: float,
    bars: int,
    closed_trades: list[float],
    final_equity: float,
    equity_curve: list[float],
) -> None:
    buy_hold = last_price / first_price
    strat_return = final_equity - 1.0
    bh_return = buy_hold - 1.0
    wins = sum(1 for r in closed_trades if r > 0)
    losses = sum(1 for r in closed_trades if r < 0)
    best = max(closed_trades) if closed_trades else 0.0
    worst = min(closed_trades) if closed_trades else 0.0
    n = len(closed_trades)
    mean_ret = sum(closed_trades) / n if n else 0.0
    var_ret = (
        sum((r - mean_ret) ** 2 for r in closed_trades) / (n - 1) if n > 1 else 0.0
    )
    sharpe = mean_ret / math.sqrt(var_ret) if var_ret > 0 else 0.0
    peak = equity_curve[0] if equity_curve else 1.0
    max_dd = 0.0
    for eq in equity_curve:
        if eq > peak:
            peak = eq
        dd = (peak - eq) / peak
        if dd > max_dd:
            max_dd = dd
    print(f"=== {name} ===")
    print(f"Bars:                  {bars}")
    print(f"Trades:                {n} (W{wins} / L{losses})")
    print(f"Strategy return:       {strat_return * 100:+.2f}%")
    print(f"Buy & Hold return:     {bh_return * 100:+.2f}%")
    print(f"Excess over BH:        {(strat_return - bh_return) * 100:+.2f}%")
    print(f"Max drawdown:          {max_dd * 100:.2f}%")
    print(
        f"Per-trade Sharpe:      {sharpe:.2f}  "
        f"(mean {mean_ret:+.4f}, stddev {math.sqrt(var_ret):.4f})"
    )
    print(f"Best / worst trade:    {best * 100:+.2f}% / {worst * 100:+.2f}%")
    print()
    print(
        "NOTE: Educational example — fees, slippage, funding costs and tax "
        "effects are simplified or omitted. Past performance is not "
        "indicative of future results."
    )


def main() -> None:
    path = Path(__file__).resolve().parents[1] / "data" / "btcusdt-1d.csv"
    candles = load_candles(path)
    if len(candles) < SQUEEZE_LOOKBACK + BB_PERIOD:
        raise SystemExit(
            f"dataset has only {len(candles)} bars; need at least "
            f"{SQUEEZE_LOOKBACK + BB_PERIOD}"
        )

    bb = ta.BollingerBands(BB_PERIOD, BB_K)
    atr = ta.ATR(ATR_PERIOD)
    bw_window: deque[float] = deque(maxlen=SQUEEZE_LOOKBACK)

    in_position = False
    entry_price = 0.0
    stop_level = 0.0
    closed_trades: list[float] = []
    equity = 1.0
    equity_curve: list[float] = []

    for c in candles:
        bb_out = bb.update(c["close"])
        atr_val = atr.update(c["high"], c["low"], c["close"])
        price = c["close"]
        mtm = equity * (price / entry_price) if in_position else equity
        equity_curve.append(mtm)

        if bb_out is None or atr_val is None:
            continue

        upper = bb_out[0] if isinstance(bb_out, tuple) else bb_out.upper
        middle = bb_out[1] if isinstance(bb_out, tuple) else bb_out.middle
        lower = bb_out[2] if isinstance(bb_out, tuple) else bb_out.lower
        bandwidth = (upper - lower) / middle if abs(middle) > 1e-12 else float("nan")

        if math.isnan(bandwidth):
            continue
        bw_window.append(bandwidth)
        if len(bw_window) < SQUEEZE_LOOKBACK:
            continue
        min_bw = min(bw_window)

        if in_position:
            stop_hit = price < stop_level
            upper_collapse = upper < entry_price
            if stop_hit or upper_collapse:
                trade_ret = price / entry_price - 1.0
                closed_trades.append(trade_ret)
                equity *= (1.0 + trade_ret) * (1.0 - FEE)
                in_position = False
        else:
            is_new_low = abs(bandwidth - min_bw) < 1e-12
            breakout = price > upper
            if is_new_low and breakout:
                entry_price = price
                stop_level = price - ATR_STOP_MULT * atr_val
                equity *= 1.0 - FEE
                in_position = True

    if in_position:
        last_price = candles[-1]["close"]
        trade_ret = last_price / entry_price - 1.0
        closed_trades.append(trade_ret)
        equity *= (1.0 + trade_ret) * (1.0 - FEE)

    print_summary(
        "Bollinger Squeeze Breakout (1d, BTCUSDT)",
        candles[0]["close"],
        candles[-1]["close"],
        len(candles),
        closed_trades,
        equity,
        equity_curve,
    )


if __name__ == "__main__":
    main()
