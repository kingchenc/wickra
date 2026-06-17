"""Strategy example: MACD crossover with ADX trend-strength filter.

Long-only trend follower. Entries fire on a MACD-line-crosses-above-
signal-line event while ADX(14) > 20 (i.e. directional market). Exits
on the opposite MACD crossover regardless of ADX. 0.1% fees per trade.

Educational example. NOT a live trading recommendation.

Run with::

    python -m examples.python.strategy_macd_adx

Uses the checked-in ``examples/data/btcusdt-1h.csv`` dataset.
"""

from __future__ import annotations

import math
from pathlib import Path

import wickra as ta

FEE = 0.001
ADX_FLOOR = 20.0


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
    path = Path(__file__).resolve().parents[1] / "data" / "btcusdt-1h.csv"
    candles = load_candles(path)

    macd = ta.MACD(12, 26, 9)
    adx = ta.ADX(14)

    in_position = False
    entry_price = 0.0
    closed_trades: list[float] = []
    equity = 1.0
    equity_curve: list[float] = []
    prev_hist_sign: bool | None = None

    for c in candles:
        macd_out = macd.update(c["close"])
        adx_out = adx.update(c)
        price = c["close"]
        mtm = equity * (price / entry_price) if in_position else equity
        equity_curve.append(mtm)

        if macd_out is None or adx_out is None:
            continue

        # MACD output is a (macd, signal, histogram) tuple/object across
        # bindings. The Python binding returns a namedtuple.
        histogram = macd_out[2] if isinstance(macd_out, tuple) else macd_out.histogram
        adx_value = adx_out[2] if isinstance(adx_out, tuple) else adx_out.adx

        hist_sign = histogram > 0.0
        cross_up = prev_hist_sign is False and hist_sign
        cross_down = prev_hist_sign is True and not hist_sign
        prev_hist_sign = hist_sign

        if not in_position and cross_up and adx_value > ADX_FLOOR:
            entry_price = price
            equity *= 1.0 - FEE
            in_position = True
        elif in_position and cross_down:
            trade_ret = price / entry_price - 1.0
            closed_trades.append(trade_ret)
            equity *= (1.0 + trade_ret) * (1.0 - FEE)
            in_position = False

    if in_position:
        last_price = candles[-1]["close"]
        trade_ret = last_price / entry_price - 1.0
        closed_trades.append(trade_ret)
        equity *= (1.0 + trade_ret) * (1.0 - FEE)

    print_summary(
        "MACD + ADX Trend Filter (1h, BTCUSDT)",
        candles[0]["close"],
        candles[-1]["close"],
        len(candles),
        closed_trades,
        equity,
        equity_curve,
    )


if __name__ == "__main__":
    main()
