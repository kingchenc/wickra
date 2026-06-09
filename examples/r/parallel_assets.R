# Run SMA(20) batch over a panel of assets, serial vs parallel::mclapply, with
# the speedup. mclapply forks on Unix; on Windows it runs serially.
library(wickra)
source("_common.R")

args <- commandArgs(trailingOnly = TRUE)
assets <- if (length(args) >= 1) as.integer(args[1]) else 200L
bars <- if (length(args) >= 2) as.integer(args[2]) else 5000L

panel <- lapply(seq_len(assets), function(a) synthetic_prices(bars, start = 50 + a * 0.1))
run_one <- function(p) { s <- Sma(20); r <- batch(s, p); r[length(r)] }

t0 <- Sys.time(); invisible(vapply(panel, run_one, numeric(1)))
serial <- as.numeric(Sys.time() - t0, units = "secs")

# mclapply forks on Unix; Windows has no fork, so it must run on a single core.
cores <- if (.Platform$OS.type == "windows") 1L else max(1L, parallel::detectCores())
t0 <- Sys.time(); invisible(parallel::mclapply(panel, run_one, mc.cores = cores))
par <- as.numeric(Sys.time() - t0, units = "secs")

cat(sprintf("%d assets x %d bars, SMA(20) batch:\n", assets, bars))
cat(sprintf("  serial   %8.1f ms\n", serial * 1000))
cat(sprintf("  parallel %8.1f ms  (%.1fx speedup, %d cores)\n",
            par * 1000, serial / max(par, 1e-9), cores))
