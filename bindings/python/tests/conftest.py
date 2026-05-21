"""Shared pytest fixtures for the Wickra Python test suite."""

from __future__ import annotations

import numpy as np
import pytest


@pytest.fixture
def linear_prices() -> np.ndarray:
    """Strictly increasing prices: 1, 2, 3, ..., 50."""
    return np.arange(1.0, 51.0, dtype=np.float64)


@pytest.fixture
def constant_prices() -> np.ndarray:
    """50 prices of 100.0."""
    return np.full(50, 100.0, dtype=np.float64)


@pytest.fixture
def sine_prices() -> np.ndarray:
    """Smooth sine-wave prices used to stress the indicators a little."""
    t = np.arange(200, dtype=np.float64)
    return 50.0 + 10.0 * np.sin(t * 0.13) + 4.0 * np.cos(t * 0.41)


@pytest.fixture
def ohlc_series() -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    """Synthetic high / low / close triple."""
    t = np.arange(200, dtype=np.float64)
    close = 100.0 + np.sin(t * 0.15) * 8.0 + np.cos(t * 0.32) * 3.0
    spread = 0.5 + np.abs(np.sin(t * 0.07))
    high = close + spread
    low = close - spread
    return high, low, close
