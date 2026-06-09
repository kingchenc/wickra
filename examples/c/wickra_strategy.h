/* Shared equity-curve summary for the Wickra C strategy examples.
 *
 * The Rust strategy examples repeat their `print_summary` per file; in C the
 * presentation is factored into this header so each strategy .c file stays
 * focused on its signal logic. Pure reporting — no indicator state.
 *
 * Header-only: define WICKRA_STRATEGY_IMPL in exactly one translation unit.
 */
#ifndef WICKRA_STRATEGY_H
#define WICKRA_STRATEGY_H

#include <stddef.h>

/* Print a one-screen summary of a strategy run: returns vs buy & hold, trade
 * win/loss counts, max drawdown, per-trade Sharpe, best/worst trade. */
void wickra_print_summary(const char *name, double first_price, double last_price,
                          size_t bars, const double *closed_trades, size_t n_trades,
                          double final_equity, const double *equity_curve,
                          size_t n_curve);

#ifdef WICKRA_STRATEGY_IMPL

#include <math.h>
#include <stdio.h>

void wickra_print_summary(const char *name, double first_price, double last_price,
                          size_t bars, const double *closed_trades, size_t n_trades,
                          double final_equity, const double *equity_curve,
                          size_t n_curve) {
    double buy_hold = last_price / first_price;
    double strat_return = final_equity - 1.0;
    double bh_return = buy_hold - 1.0;

    size_t wins = 0, losses = 0;
    double best = -INFINITY, worst = INFINITY;
    double sum_ret = 0.0, sum_sq = 0.0;
    for (size_t i = 0; i < n_trades; ++i) {
        double r = closed_trades[i];
        if (r > 0.0) {
            wins++;
        } else if (r < 0.0) {
            losses++;
        }
        if (r > best) {
            best = r;
        }
        if (r < worst) {
            worst = r;
        }
        sum_ret += r;
        sum_sq += r * r;
    }
    double n = (double)n_trades;
    double mean_ret = n > 0.0 ? sum_ret / n : 0.0;
    double var_ret = n > 1.0 ? (sum_sq - n * mean_ret * mean_ret) / (n - 1.0) : 0.0;
    double sharpe = var_ret > 0.0 ? mean_ret / sqrt(var_ret) : 0.0;
    if (n_trades == 0) {
        best = 0.0;
        worst = 0.0;
    }

    double peak = n_curve > 0 ? equity_curve[0] : 1.0;
    double max_dd = 0.0;
    for (size_t i = 0; i < n_curve; ++i) {
        if (equity_curve[i] > peak) {
            peak = equity_curve[i];
        }
        double dd = (peak - equity_curve[i]) / peak;
        if (dd > max_dd) {
            max_dd = dd;
        }
    }

    printf("=== %s ===\n", name);
    printf("Bars:                  %llu\n", (unsigned long long)bars);
    printf("Trades:                %llu (W%llu / L%llu)\n", (unsigned long long)n_trades,
           (unsigned long long)wins, (unsigned long long)losses);
    printf("Strategy return:       %+.2f%%\n", strat_return * 100.0);
    printf("Buy & Hold return:     %+.2f%%\n", bh_return * 100.0);
    printf("Excess over BH:        %+.2f%%\n", (strat_return - bh_return) * 100.0);
    printf("Max drawdown:          %.2f%%\n", max_dd * 100.0);
    printf("Per-trade Sharpe:      %.2f  (mean %+.4f, stddev %.4f)\n", sharpe,
           mean_ret, sqrt(var_ret));
    printf("Best / worst trade:    %+.2f%% / %+.2f%%\n", best * 100.0, worst * 100.0);
    printf("\n");
    printf("NOTE: Educational example — fees, slippage, funding costs and tax "
           "effects are simplified or omitted. Past performance is not indicative "
           "of future results.\n");
}

#endif /* WICKRA_STRATEGY_IMPL */
#endif /* WICKRA_STRATEGY_H */
