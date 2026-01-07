# Percolator Security Audit - Final Status

## Executive Summary

| Category | Count | Status |
|----------|-------|--------|
| Actually Critical | 4 | **All Fixed** |
| Real (Lower Impact) | 5 | **All Fixed/Mitigated** |
| Not Real Bugs | 11 | N/A |

**Conclusion:** No must-fix issues remaining. All exploitable vulnerabilities have been addressed.

---

## Actually Critical (Exploitable) - ALL FIXED

### #1: TradeNoCpi Oracle Substitution
**Status:** FIXED (line 1983)
**Severity:** CRITICAL
**Why Real:** Attacker could substitute different Pyth feed, manipulate trade execution prices, induce wrongful liquidations.
**Fix:** Added `oracle_key_ok(config.index_oracle, a_oracle.key.to_bytes())` check.

### #7: Oracle Parser Accepts Wrong Pyth Account Types
**Status:** FIXED (lines 1248-1256)
**Severity:** CRITICAL
**Why Real:** Could parse Pyth mapping/product accounts as price, yielding garbage values.
**Fix:** Added validation for Pyth magic (0xa1b2c3d4), version (2), account_type (3=Price).

### #9/#10: Oracle Exponent Overflow / Div-by-Zero
**Status:** FIXED (line 1263)
**Severity:** CRITICAL
**Why Real:** Extreme exponents could overflow `10u128.pow()`, wrap to 0, cause division by zero or bogus prices.
**Fix:** Added `MAX_EXPO_ABS = 18` bound. Exponents outside [-18, +18] rejected.

### #12: Permissionless allow_panic
**Status:** FIXED (line 1858)
**Severity:** CRITICAL
**Why Real:** Anyone could trigger global settlement during stress - major griefing/manipulation vector.
**Fix:** `allow_panic != 0` now requires admin signature via `admin_ok()` check.

---

## Real But Lower Impact - ALL FIXED/MITIGATED

### #3: InitMarket collateral_mint Mismatch
**Status:** FIXED (line 1593)
**Severity:** MEDIUM (admin-only)
**Why Real:** Data vs account mismatch could confuse transaction reviewers.
**Fix:** Added `collateral_mint != *a_mint.key` validation.

### #5/#6: InitMarket Mint/Vault Validation
**Status:** FIXED (lines 1599-1612, 1515-1517)
**Severity:** MEDIUM (admin-only)
**Why Real:** Invalid mint/vault could brick market. But admin would brick their own market.
**Fix:** Added SPL Mint validation and vault `AccountState::Initialized` check.

### #8: Oracle No Status Check
**Status:** FIXED (lines 1271-1274)
**Severity:** HIGH
**Why Real:** Halted/invalid Pyth prices could cause bad liquidations.
**Fix:** Added `status == PYTH_STATUS_TRADING` check (devnet-gated).

### #13: Funding Accrual Errors Ignored (Engine)
**Status:** FIXED
**Severity:** HIGH
**Why Real:** Could permanently wedge funding if errors silently ignored.
**Fix:** Engine now propagates errors.

### #14: Liquidation Errors Swallowed (Engine)
**Status:** MITIGATED
**Severity:** MEDIUM
**Why Mitigated:** Fail-safe pattern now assumes max loss on error instead of silently skipping. Liquidation still happens.

---

## Not Real Security Bugs

### #2: CLI --oracle UX
**Status:** NOT A BUG
**Why:** CLI UX issue, not program vulnerability. Fix #1 makes this irrelevant - program rejects wrong oracle regardless of what CLI passes.

### #4: InitMarket Oracle Accounts Unused
**Status:** NOT A BUG
**Why:** Design choice. Program stores oracle pubkeys from instruction data, not account metas. This is valid - instruction data is what gets signed.

### #11: Devnet Feature Disables Checks
**Status:** NOT A BUG
**Why:** Intentional for testing. Mainnet builds exclude `--features devnet`. Standard practice.

### #15: ADL O(n²)
**Status:** FIXED (was real, now fixed)
**Why Fixed:** Now uses heap-based O(n log n) algorithm.

### #16/#17: Saturating Arithmetic
**Status:** NOT A BUG
**Why:** Theoretical concern. Values bounded by position limits. Saturation is safer than panic - produces conservative results rather than crashing.

### #18/#19: Oracle Bypass Variants
**Status:** DUPLICATES
**Why:** Just restatements of #1. Fixed by #1 fix.

### #20: ABI Drift (Unused Accounts)
**Status:** FIXED
**Why:** Was cleanup issue, not security. InitUser/InitLP now correctly use 5 accounts.

---

## Fix Summary

| Issue | Fix | Location |
|-------|-----|----------|
| #1: Oracle substitution | oracle_key_ok | 1983 |
| #7: Pyth type validation | magic/version/type | 1248-1256 |
| #9/#10: Exponent overflow | MAX_EXPO_ABS=18 | 1263 |
| #12: allow_panic auth | admin_ok | 1858 |
| #3: collateral_mint | equality check | 1593 |
| #5/#6: Mint/vault validation | SPL checks | 1599-1612, 1515-1517 |
| #8: Pyth status | status==1 | 1271-1274 |
| #13: Funding errors | propagate errors | engine |
| #14: Liq errors | fail-safe pattern | engine |
| #15: ADL complexity | heap-based | engine |
| #20: ABI drift | 5 accounts | 1666, 1695 |

---

## Audit Assessment

The original audit identified 20 findings but overcounted by:
- Treating duplicates as separate issues (#18, #19 = #1)
- Classifying design choices as bugs (#4, #11)
- Listing theoretical concerns as high severity (#16, #17)
- Including CLI UX issues as program vulnerabilities (#2)

**Actual critical vulnerabilities: 4** (all fixed)
**Actual high/medium issues: 5** (all fixed/mitigated)
**False positives or low-value findings: 11**

---

## Benchmark Validation

| Scenario | Worst CU | % Limit | Liquidations |
|----------|----------|---------|--------------|
| Baseline | 16,314 | 1.2% | - |
| 4095 healthy | 39,684 | 2.8% | - |
| 4095 + crash | 54,533 | 3.9% | 512 force |
| MTM worst case | 740,166 | 52.9% | 2047 |

All scenarios under 1.4M CU limit.
