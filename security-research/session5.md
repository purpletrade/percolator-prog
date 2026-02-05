# Security Research Log - 2026-02-05 Session 5

## Continued Systematic Search

### Areas Verified This Session

#### 1. SetRiskThreshold ✓
**Location**: `percolator-prog/src/percolator.rs:3290-3307`
**Status**: SECURE

- Admin-only (require_admin check)
- No parameter validation needed (any u128 is valid)
- Used to control force-realize mode activation

#### 2. Force-Realize Mode ✓
**Location**: `percolator/src/percolator.rs:1453-1458, 1610-1628`
**Status**: SECURE

- Triggers when `insurance_fund.balance <= risk_reduction_threshold`
- Uses touch_account_for_force_realize (best-effort fee settle)
- Closes positions at oracle price
- Maintains lifetime counter with saturating_add

#### 3. Partial Position Close ✓
**Location**: `percolator/src/percolator.rs:1792-1871`
**Status**: SECURE

- Zero checks for close_abs and current_abs_pos
- Falls back to full close when close_abs >= current_abs_pos
- Uses checked_mul/checked_div with fallback
- Maintains OI and LP aggregates
- Settles warmup and writes off negative PnL

#### 4. Entry Price Updates ✓
**Location**: Multiple (1906, 2258, 2278, 2292, 2308, 3022, 3028)
**Status**: SECURE

- Entry price always set to oracle_price after mark settlement
- Ensures mark_pnl = 0 at settlement price
- Consistent across all settlement paths

#### 5. LiquidateAtOracle Instruction ✓
**Location**: `percolator-prog/src/percolator.rs:3123-3178`
**Status**: SECURE

- Permissionless (anyone can liquidate underwater accounts)
- Uses check_idx before account access
- Proper oracle price retrieval and circuit breaker

## Running Verification

All 57 integration tests pass.

#### 6. Dust Sweeping ✓
**Location**: `percolator-prog/src/percolator.rs:2752-2812`
**Status**: SECURE

- Dust accumulated from base_to_units remainders
- Swept to insurance fund when accumulated >= unit_scale
- CloseSlab checks dust_base != 0 (Bug #3 fix)
- Uses saturating_add for accumulation

#### 7. Unit Scale Handling ✓
**Location**: `percolator-prog/src/percolator.rs:729-736`
**Status**: SECURE

- scale_price_e6 rejects zero result
- MAX_UNIT_SCALE = 1 billion
- Withdrawal rejects misaligned amounts
- Ensures oracle values match capital scale

#### 8. Warmup Reset ✓
**Location**: `percolator/src/percolator.rs:2809-2813, 3084-3085`
**Status**: SECURE

- update_warmup_slope called when avail_gross increases
- Called after trade for both parties
- Resets warmup_started_at_slot to current_slot
- Slope = avail_gross / warmup_period_slots (min=1 if avail>0)

#### 9. Funding Index Overflow ✓
**Location**: `percolator/src/percolator.rs:2144-2147`
**Status**: SECURE

- Uses checked_add for funding_index_qpb_e6 update
- Returns Overflow error on overflow
- Rate capped at ±10,000 bps/slot
- dt capped at ~1 year (31,536,000 slots)

#### 10. Fee Credits ✓
**Location**: `percolator/src/percolator.rs:1049-1067`
**Status**: SECURE

- Starts at 0, deducted by maintenance fees
- Can go negative (fees owed)
- Paid from capital when negative
- Uses saturating_sub/saturating_add
- Forgiven on close_account (Finding C fix)

#### 11. Reserved PnL ✓
**Location**: `percolator/src/percolator.rs:119, 2039, 2064`
**Status**: SECURE

- Subtracted from positive PnL to get available gross
- Must be zero for GC
- Uses saturating_sub for safety
- Prevents claiming reserved PnL early

#### 12. CloseAccount ✓
**Location**: `percolator/src/percolator.rs:1261-1324`
**Status**: SECURE

- Full settlement via touch_account_full
- Position must be zero
- Fee debt forgiven (Finding C fix)
- PnL must be exactly 0
- Capital verified against vault
- c_tot updated before free_slot

## Session 5 Summary

**Additional Areas Verified**: 12
**New Vulnerabilities Found**: 0
**Test Status**: All 57 integration tests pass

The systematic search continues to find no new vulnerabilities. The codebase demonstrates comprehensive security measures across all reviewed areas.
