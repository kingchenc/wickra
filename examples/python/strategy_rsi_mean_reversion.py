"""Strategy example: RSI mean-reversion on hourly BTCUSDT data.

Goes long when RSI(14) crosses below 30 (oversold), exits when RSI
crosses above 70 (overbought). Position is binary (full-in / full-out),
fees are 0.1% per trade (Binance maker tier), no stop-loss.

Educational example. NOT a recommended trading strategy in real markets.
The point is to show how Wickra streaming indicators wire up into a
complete signal -> fill -> PnL -> equity loop in a single file.

Run with::

    python -m examples.python.strategy_rsi_mean_reversion

Uses the checked-in ``examples/data/btcusdt-1h.csv`` dataset.
"""

from __future__ import annotations

import csv
import math
from pathlib import Path

import wickra as ta

FEE = 0.001
RSI_PERIOD = 14
OVERSOLD = 30.0
OVERBOUGHT = 70.0


def load_candles(path: Path) -> list[dict[str, float]]:
    with path.open() as fh:
        reader = csv.DictReader(fh)
        return [
            {
                "open": float(r["open"]),
                "high": float(r["high"]),
                "low": float(r["low"]),
                "close": float(r["close"]),
                "volume": float(r["volume"]),
            }
            for r in reader
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
    if len(candles) < RSI_PERIOD * 4:
        raise SystemExit(f"dataset too small: {len(candles)}")

    rsi = ta.RSI(RSI_PERIOD)

    in_position = False
    entry_price = 0.0
    closed_trades: list[float] = []
    equity = 1.0
    equity_curve: list[float] = []

    for c in candles:
        rsi_val = rsi.update(c["close"])
        price = c["close"]
        mtm = equity * (price / entry_price) if in_position else equity
        equity_curve.append(mtm)

        if rsi_val is None:
            continue

        if not in_position and rsi_val < OVERSOLD:
            entry_price = price
            equity *= 1.0 - FEE
            in_position = True
        elif in_position and rsi_val > OVERBOUGHT:
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
        "RSI Mean-Reversion (1h, BTCUSDT)",
        candles[0]["close"],
        candles[-1]["close"],
        len(candles),
        closed_trades,
        equity,
        equity_curve,
    )


if __name__ == "__main__":
    main()
