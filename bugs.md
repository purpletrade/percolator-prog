# Percolator Security Audit - First Principles Review

Auditor perspective: Assume developer is adversarial. Look for backdoors, fund extraction, manipulation vectors.

## Executive Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 4 | **All Verified Fixed** |
| HIGH | 8 | **All Fixed/Mitigated** |
| MEDIUM | 2 | Mitigated |
| INFO | 1 | ABI Drift (non-exploitable) |

**Key Finding**: No direct admin backdoors for fund extraction. Primary attack vectors were oracle substitution and system wedging - now fixed in program layer. Engine fail-safe patterns prevent overflow-based wedging.

---

## ADVERSARIAL REVIEW (2026-01-07)

Independent verification of all claimed fixes. Checked for:
- Unreported vulnerabilities
- Incorrect fix claims
- Backdoors or fund extraction vectors
- Logic bugs in margin/liquidation
- Integer overflow/underflow paths

**Result**: All claimed fixes verified. No new critical issues found.

---

## CRITICAL ISSUES - ALL VERIFIED FIXED

### C1: VERIFIED FIXED - TradeNoCpi Oracle Substitution
**Location**: `percolator-prog/src/percolator.rs:1983`
**Verification**: `oracle_key_ok(config.index_oracle, a_oracle.key.to_bytes())` check confirmed present before `read_pyth_price_e6()`.

### C2: VERIFIED FIXED - Oracle Parser Accepts Wrong Pyth Account Types
**Location**: `percolator-prog/src/percolator.rs:1244-1256`
**Verification**: Confirmed checks for:
- `magic != PYTH_MAGIC` (0xa1b2c3d4)
- `version != PYTH_VERSION_2` (2)
- `account_type != PYTH_ACCOUNT_TYPE_PRICE` (3)

### C3: VERIFIED FIXED - Oracle Exponent Overflow
**Location**: `percolator-prog/src/percolator.rs:1263`
**Verification**: `expo.abs() > MAX_EXPO_ABS` (18) check confirmed.

### C4: VERIFIED FIXED - Permissionless allow_panic
**Location**: `percolator-prog/src/percolator.rs:1856-1859`
**Verification**: `admin_ok(header.admin, a_caller.key.to_bytes())` check confirmed when `allow_panic != 0`.

---

## HIGH ISSUES - ALL VERIFIED FIXED/MITIGATED

### H1: VERIFIED FIXED - InitMarket Data/Account Mismatch
**Location**: `percolator-prog/src/percolator.rs:1593`
**Verification**: `collateral_mint != *a_mint.key` equality check confirmed.

### H2: VERIFIED FIXED - InitMarket No SPL Mint Validation
**Location**: `percolator-prog/src/percolator.rs:1599-1612`
**Verification**: Confirmed checks for:
- `*a_mint.owner != spl_token::ID`
- `a_mint.data_len() != Mint::LEN`
- `Mint::unpack(&mint_data)` validation

### H3: VERIFIED FIXED - verify_vault No State Check
**Location**: `percolator-prog/src/percolator.rs:1515-1517`
**Verification**: `tok.state != AccountState::Initialized` check confirmed.

### H4: VERIFIED FIXED - Oracle No Status Check
**Location**: `percolator-prog/src/percolator.rs:1271-1274`
**Verification**: `status != PYTH_STATUS_TRADING` (1) check confirmed (devnet-gated).

### H5: VERIFIED FIXED - Devnet Feature Safety
**Location**: `percolator-prog/src/percolator.rs:1191-1197`
**Verification**: Security warning comment confirmed documenting devnet risks.

### H6: MITIGATED - Liquidation Error Swallowing
**Location**: `percolator/src/percolator.rs:2435-2439`
**Status**: MITIGATED
**Analysis**:
- Errors now set `risk_reduction_only = true` (line 2438)
- All close helpers now use fail-safe patterns (commit 330779e)
- Overflow errors in mark_pnl are caught and treated as worst-case loss
- **Residual risk**: None significant - fail-safe pattern prevents wedging

### H7: INFO - ABI Drift (Unused Accounts)
**Location**: `percolator-cli/src/abi/accounts.ts` vs `percolator-prog/src/percolator.rs`
**Status**: NON-EXPLOITABLE (downgraded to INFO)
**Analysis**:
- InitUser/InitLP expect 7 accounts (line 1666, 1695) but only use indices 0-4
- ABI declares clock (index 5) and oracle (index 6) but handlers ignore them
- **Impact**: Wasteful transaction size, confusing ABI, no security impact
- **Recommendation**: Either remove unused accounts from ABI or add validation in handler

### H8: BY DESIGN - Socialization Wedge Risk
**Location**: `percolator/src/percolator.rs:2669-2673`
**Status**: BY DESIGN (downgraded)
**Analysis**:
- If `pending_profit_to_fund > 0` cannot be covered by insurance, withdrawals block
- This is **intentional**: prevents extraction of unfunded profit
- `finalize_pending_after_window()` runs every full sweep (line 1493)
- Keeper liveness required, but this is standard for any keeper-dependent system
- **Not a bug**: Correct behavior to prevent insolvency

---

## MEDIUM ISSUES - MITIGATED

### M1: MITIGATED - Saturating Arithmetic Edge Cases
**Location**: `percolator/src/percolator.rs`
**Status**: MITIGATED
**Analysis**:
- mark_pnl overflow now uses fail-safe pattern everywhere
- `u128_to_i128_clamped()` prevents wrap-around in worst-case fallbacks
- Commit 330779e addressed all remaining i128 cast issues

### M2: LOW RISK - O(n) ADL Remainder Distribution
**Location**: `percolator/src/percolator.rs:2546-2604`
**Status**: LOW RISK (severity reduced)
**Analysis**:
- socialization_step is O(WINDOW) not O(n^2)
- Window size is bounded per crank
- CU budget verified: worst case 740K CU (52.9% of limit)

---

## First-Principles Security Analysis

### Can Admin Steal Funds?
**NO** - No instruction allows admin to withdraw vault or insurance directly.
- Admin can only: SetRiskThreshold, UpdateAdmin, allow_panic (triggers settlement)
- Verified: No hidden transfer paths

### Can LP Steal from Users?
**NO (Fixed)** - Oracle substitution attack blocked by C1 fix.
- Verified: oracle_key_ok check at line 1983

### Can Users Steal from LP?
**NO (Fixed)** - Same oracle substitution fix.

### Can System Be Wedged?
**NO (Fixed)** - All forced-close paths are fail-safe.
- Verified: Commits 330779e, 92d3e5d address all overflow paths
- mark_pnl overflow -> worst-case loss (not error)
- Liquidation always succeeds (may assume max loss)

### Are Funds Extractable Without Authorization?
**NO** - All token transfers require owner signature + PDA authority.
- Verified: owner_ok checks on all user operations

### Is There a Rug Pull Vector?
**NO** - Admin cannot drain funds or modify positions.
- Admin burn (zero pubkey) locks out admin operations permanently

---

## Verified Fix Summary

| Issue | Fix | Line | Verified |
|-------|-----|------|----------|
| C1: Oracle substitution | oracle_key_ok | 1983 | YES |
| C2: Pyth type validation | magic/version/type | 1244-1256 | YES |
| C3: Exponent overflow | MAX_EXPO_ABS | 1263 | YES |
| C4: allow_panic auth | admin_ok | 1856-1859 | YES |
| H1: collateral_mint | equality check | 1593 | YES |
| H2: SPL Mint validation | owner/unpack | 1599-1612 | YES |
| H3: Vault state | Initialized check | 1515-1517 | YES |
| H4: Pyth status | status == 1 | 1271-1274 | YES |
| H5: Devnet warning | comment | 1191-1197 | YES |
| H6: Liq errors | fail-safe pattern | engine | YES |
| H8: Socialization | by design | engine | N/A |

---

## Engine Fail-Safe Commits

| Commit | Description |
|--------|-------------|
| 330779e | Make all close helpers fail-safe, fix i128 cast |
| 92d3e5d | Add fail-safe guards for corrupted account state |
| 877ad48 | Complete bounds enforcement |
| 8fdb96f | Prevent overflow in mark_pnl, enforce CU budget |
| 261008a | Fix wedge + fail-safe bugs in MTM |
| 1789243 | MTM risk correctness |

---

## Benchmark Validation

| Scenario | Worst CU | % Limit | Liquidations |
|----------|----------|---------|--------------|
| Baseline | 16,314 | 1.2% | - |
| 4095 healthy | 39,684 | 2.8% | - |
| 4095 + crash | 54,533 | 3.9% | 512 force |
| **MTM worst case** | **740,166** | **52.9%** | **2047** |

All scenarios under 1.4M CU limit. MTM liquidations working correctly.
