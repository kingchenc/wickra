---
name: Feature request (Detailed)
about: Long-form proposal with API sketch, scope checkboxes, prior-art links, and contribution intent.
title: "[Feat] <short description>"
labels: ["enhancement", "triage"]
assignees: []
---

## Problem / motivation

<!--
What are you trying to do that Wickra doesn't support today?
Describe the user-facing pain point, not the implementation.
-->

## Proposed solution

<!--
Sketch the API or behavior you'd like. A short code snippet of how
you'd want to call it is worth a thousand words.
-->

```python
import wickra as ta

# proposed API
ind = ta.SuperTrend(period=10, multiplier=3.0)
ind.update(close, high, low)
```

## Scope

- [ ] New indicator
- [ ] New method on an existing indicator
- [ ] New binding / platform target
- [ ] Performance improvement
- [ ] Ergonomics / API cleanup
- [ ] Other (Explain below)

## Reference / prior art

<!--
Link the paper, book chapter, TA-Lib function, TradingView Pine source,
or other implementations you'd like Wickra to match.
-->

## Alternatives considered

<!-- What workarounds exist today? Why aren't they enough? -->

## Willingness to contribute

- [ ] I'd like to implement this myself with guidance
- [ ] I can help review / test
- [ ] Requesting only — no bandwidth to implement
