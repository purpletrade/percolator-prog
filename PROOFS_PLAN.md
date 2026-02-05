# Kani Proofs Plan: Proving Aggregate Safety Class

## Goal

Prove the **ENTIRE CLASS** of aggregate consistency bugs is impossible.

Bug #10 was caused by code that modified `acc.pnl` directly instead of using `set_pnl()`.
This class of bug can occur anywhere PnL or capital is modified. The proofs below
ensure that **ALL code paths** maintain aggregate invariants.

## Critical Bugs Found by Security Sweeps

| Bug | Class | Root Cause | Proof Strategy |
|-----|-------|------------|----------------|
| **#10** | Aggregate Desync | Direct pnl assignment bypassed set_pnl() | Prove pnl_pos_tot invariant holds after ALL operations |
| **#9** | Rate Limiting | clamp_toward_with_dt returned mark when dt=0 | Prove index movement bounded per slot |

## The Aggregate Invariant Class

**Invariant A1: pnl_pos_tot Consistency**
```
∀ engine states: engine.pnl_pos_tot == Σ max(account[i].pnl, 0)
```

**Invariant A2: c_tot Consistency**
```
∀ engine states: engine.c_tot == Σ account[i].capital
```

**Invariant A3: Conservation**
```
∀ engine states: vault >= c_tot + insurance + haircut_adjusted_positive_pnl
```

## Bug #10 Details

The force-close code originally did:
```rust
// VULNERABLE (before fix):
let old_pnl = acc.pnl.get();
acc.pnl = percolator::I128::new(old_pnl.saturating_add(pnl_delta));
```

This bypassed `set_pnl()` which maintains the `pnl_pos_tot` aggregate. The fix uses:
```rust
// FIXED:
engine.set_pnl(idx as usize, new_pnl);
```

**Why this bug class is dangerous:**
- `haircut_ratio()` uses `pnl_pos_tot` for withdrawal calculations
- Stale aggregate → incorrect haircut → value extraction

## Proving the Entire Bug Class Safe

The key insight: **Every public method that modifies state must preserve invariants.**

Instead of proving individual bugs don't exist, we prove:
1. The invariant holds initially (engine construction)
2. Every operation preserves the invariant (inductive step)
3. Therefore, the invariant holds for ALL reachable states

### Operations That Modify PnL (Must Use set_pnl)

| Operation | Location | Modifies PnL | Uses set_pnl |
|-----------|----------|--------------|--------------|
| execute_trade | RiskEngine | ✓ | ✓ |
| settle_account_funding | RiskEngine | ✓ | ✓ |
| settle_mark_to_oracle | RiskEngine | ✓ | ✓ |
| settle_warmup_to_capital | RiskEngine | ✓ | ✓ |
| force_realize | RiskEngine | ✓ | ✓ |
| oracle_close_position | RiskEngine | ✓ | ✓ |
| **force-close (program)** | percolator.rs | ✓ | ✓ (FIXED) |

### Operations That Modify Capital (Must Use set_capital)

| Operation | Location | Modifies Capital | Uses set_capital |
|-----------|----------|------------------|------------------|
| deposit | RiskEngine | ✓ | ✓ |
| withdraw | RiskEngine | ✓ | ✓ |
| execute_trade (fees) | RiskEngine | ✓ | ✓ |
| settle_warmup_to_capital | RiskEngine | ✓ | ✓ |
| charge_maintenance_fee | RiskEngine | ✓ | ✓ |
| liquidate_at_oracle | RiskEngine | ✓ | ✓ |

## Proof Categories

### Category 1: Aggregate Consistency (Proves Entire Bug Class Safe)

**Proof 1.1: set_pnl Maintains Invariant (Foundation)**
```rust
#[kani::proof]
fn set_pnl_maintains_pnl_pos_tot_invariant() {
    let mut engine = RiskEngine::new(test_params());
    let idx = engine.add_user(0).unwrap();

    // Precondition: invariant holds
    kani::assume(check_pnl_pos_tot_invariant(&engine));

    let new_pnl: i128 = kani::any();
    kani::assume(new_pnl > -1_000_000 && new_pnl < 1_000_000);

    engine.set_pnl(idx as usize, new_pnl);

    // Postcondition: invariant still holds
    assert!(check_pnl_pos_tot_invariant(&engine),
        "set_pnl must maintain pnl_pos_tot invariant");
}
```

**Proof 1.2: EVERY Operation Preserves pnl_pos_tot**
```rust
// One proof per operation that could affect PnL:

#[kani::proof]
fn deposit_preserves_pnl_pos_tot() { ... }

#[kani::proof]
fn withdraw_preserves_pnl_pos_tot() { ... }

#[kani::proof]
fn execute_trade_preserves_pnl_pos_tot() { ... }

#[kani::proof]
fn settle_funding_preserves_pnl_pos_tot() { ... }

#[kani::proof]
fn settle_warmup_preserves_pnl_pos_tot() { ... }

#[kani::proof]
fn liquidate_preserves_pnl_pos_tot() { ... }

#[kani::proof]
fn close_account_preserves_pnl_pos_tot() { ... }

// ... one for EACH public method
```

**Proof 1.3: c_tot Invariant (Same Pattern)**
```rust
#[kani::proof]
fn set_capital_maintains_c_tot_invariant() {
    // Same pattern as pnl_pos_tot
    // Proves c_tot == sum(account.capital) after set_capital
}
```

**Proof 1.4: Combined Aggregate Invariant**
```rust
#[kani::proof]
fn all_aggregates_consistent() {
    // After ANY sequence of operations:
    // Assert: pnl_pos_tot == computed
    // Assert: c_tot == computed
    // Assert: conservation holds
}
```

### Category 2: Force-Close Specific Proofs

**Proof 2.1: Force-Close PnL Calculation Bounds**
```rust
#[kani::proof]
fn force_close_pnl_bounded() {
    // For any position, entry_price, settlement_price:
    // pnl_delta = pos * (settle - entry) / 1e6
    // Assert: No overflow in saturating arithmetic
    // Assert: Result is bounded by position * max_price_delta
}
```

**Proof 2.2: Force-Close Preserves Conservation**
```rust
#[kani::proof]
fn force_close_preserves_conservation() {
    // Setup: Engine with open positions
    // Action: Simulate force-close settlement
    // Assert: vault >= c_tot + insurance + haircut_adjusted_pnl
}
```

**Proof 2.3: Force-Close Zeroes Position**
```rust
#[kani::proof]
fn force_close_zeroes_position() {
    // Setup: Account with non-zero position
    // Action: Force-close at settlement price
    // Assert: position_size == 0 after settlement
    // Assert: entry_price == 0 after settlement
}
```

### Category 3: Haircut Ratio Correctness

**Proof 3.1: Haircut Ratio Uses Accurate pnl_pos_tot**
```rust
#[kani::proof]
fn haircut_uses_accurate_pnl_pos_tot() {
    // Setup: Engine with various PnL distributions
    // Compute: expected_pnl_pos_tot from account iteration
    // Assert: engine.pnl_pos_tot == expected_pnl_pos_tot
    // Assert: haircut_ratio() returns correct value
}
```

**Proof 3.2: Haircut Cannot Exceed 1**
```rust
#[kani::proof]
fn haircut_ratio_bounded() {
    // For any engine state:
    // (h_num, h_den) = engine.haircut_ratio()
    // Assert: h_num <= h_den (ratio <= 1)
}
```

**Proof 3.3: Effective PnL With Haircut**
```rust
#[kani::proof]
fn effective_pnl_with_haircut_bounded() {
    // For any pnl and engine state:
    // eff = engine.effective_pos_pnl(pnl)
    // Assert: eff <= max(pnl, 0) (haircut only reduces, never increases)
}
```

### Category 4: State Transition Invariants

**Proof 4.1: Resolved Market Blocks Trading**
```rust
#[kani::proof]
fn resolved_market_blocks_trade() {
    // Setup: Resolved market (simulated via flag)
    // Action: Attempt trade
    // Assert: Returns error (trading blocked)
}
```

**Proof 4.2: Insurance Withdrawal Requires Positions Closed**
```rust
#[kani::proof]
fn insurance_withdrawal_requires_no_positions() {
    // Setup: Market with open positions
    // Action: Attempt insurance withdrawal
    // Assert: Fails if any position != 0
}
```

### Category 5: Pagination Correctness

**Proof 5.1: Crank Cursor Bounds**
```rust
#[kani::proof]
fn crank_cursor_bounded() {
    // For any cursor value and batch processing:
    // Assert: cursor always < MAX_ACCOUNTS
    // Assert: cursor wraps correctly at boundary
}
```

**Proof 5.2: All Accounts Eventually Processed**
```rust
#[kani::proof]
fn pagination_covers_all_accounts() {
    // Setup: N accounts with positions
    // Action: Simulate ceil(N/BATCH_SIZE) cranks
    // Assert: All accounts have been visited
}
```

## Why These Proofs Catch the ENTIRE Bug Class

### The Inductive Argument

1. **Base Case**: `RiskEngine::new()` initializes aggregates correctly
   - `pnl_pos_tot = 0` (no accounts)
   - `c_tot = 0` (no capital)
   - Invariant holds trivially

2. **Inductive Step**: Every public method preserves invariants
   - If invariant holds before operation, it holds after
   - Proven separately for EACH method

3. **Conclusion**: Invariant holds for ALL reachable states
   - Any code path through the engine maintains aggregates
   - Bug #10 would be caught because force-close is a reachable operation

### What This Proves

| Property | Meaning |
|----------|---------|
| **No Stale Aggregates** | Every PnL/capital change updates aggregates |
| **Correct Haircut** | haircut_ratio() always uses accurate pnl_pos_tot |
| **No Value Extraction** | Withdrawals limited by accurate effective_equity |
| **Future-Proof** | New code must pass proofs to be merged |

### Coverage Guarantee

By proving EVERY public method individually:
- `add_user`, `add_lp`
- `deposit`, `withdraw`
- `execute_trade`
- `settle_account_funding`
- `settle_mark_to_oracle`, `settle_mark_to_oracle_best_effort`
- `settle_warmup_to_capital`
- `force_realize`
- `liquidate_at_oracle`
- `oracle_close_position`, `oracle_close_position_core`
- `close_account`
- `charge_maintenance_fee`
- **Any new method must be added to proof suite**

## Implementation Strategy

### Phase 1: Core Aggregate Proofs (CRITICAL - Catches Bug Class)
These prove the invariant is maintained by the helpers:
1. `set_pnl_maintains_pnl_pos_tot_invariant`
2. `set_capital_maintains_c_tot_invariant`
3. `recompute_aggregates_correct`

### Phase 2: Operation-Level Proofs (Inductive Step)
One proof per public method showing invariant preservation:
4. `deposit_preserves_aggregates`
5. `withdraw_preserves_aggregates`
6. `execute_trade_preserves_aggregates`
7. `settle_funding_preserves_aggregates`
8. `settle_warmup_preserves_aggregates`
9. `liquidate_preserves_aggregates`
10. `close_account_preserves_aggregates`
11. `force_realize_preserves_aggregates`

### Phase 3: Haircut Correctness (Depends on Aggregates)
12. `haircut_ratio_bounded` - ratio in [0, 1]
13. `haircut_uses_accurate_pnl_pos_tot` - no stale data
14. `effective_pnl_bounded_by_actual` - haircut only reduces

### Phase 4: Program-Level Proofs (Force-Close)
These prove the Solana program code (not just engine):
15. `force_close_uses_set_pnl` - static analysis / wrapper test
16. `force_close_preserves_conservation`
17. `crank_cursor_bounded`

### Phase 5: Rate Limiting (Bug #9 Class)
18. `index_movement_bounded_per_slot`
19. `clamp_toward_no_movement_when_dt_zero`

## Helper Functions Needed

```rust
/// Compute pnl_pos_tot by iterating all accounts
fn computed_pnl_pos_tot(engine: &RiskEngine) -> u128 {
    let mut sum = 0u128;
    for i in 0..MAX_ACCOUNTS {
        if engine.is_used(i) {
            let pnl = engine.accounts[i].pnl.get();
            if pnl > 0 {
                sum = sum.saturating_add(pnl as u128);
            }
        }
    }
    sum
}

/// Check aggregate consistency invariant
fn check_pnl_pos_tot_invariant(engine: &RiskEngine) -> bool {
    engine.pnl_pos_tot.get() == computed_pnl_pos_tot(engine)
}

/// Simulate force-close for a single account
fn simulate_force_close(
    engine: &mut RiskEngine,
    idx: usize,
    settlement_price: u64,
) {
    let acc = &engine.accounts[idx];
    let pos = acc.position_size.get();
    if pos != 0 {
        let entry = acc.entry_price as i128;
        let settle = settlement_price as i128;
        let pnl_delta = pos.saturating_mul(settle - entry) / 1_000_000;
        let old_pnl = acc.pnl.get();
        let new_pnl = old_pnl.saturating_add(pnl_delta);
        engine.set_pnl(idx, new_pnl);  // MUST use set_pnl
        engine.accounts[idx].position_size = I128::ZERO;
        engine.accounts[idx].entry_price = 0;
    }
}
```

## Kani Configuration

```rust
// Recommended unwind bounds for premarket proofs
#[kani::unwind(33)]  // Standard for most proofs
#[kani::solver(cadical)]  // Fast SAT solver

// For pagination proofs with more accounts:
#[kani::unwind(65)]  // BATCH_SIZE + 1
```

## Success Criteria

### Proving Bug Class Impossible

1. **Completeness**: Every public method has an aggregate preservation proof
2. **Soundness**: Proofs pass on current (fixed) code
3. **Sensitivity**: Proofs would FAIL on pre-fix code (Bug #10)
4. **No False Positives**: Proofs don't reject valid code patterns

### Regression Prevention

5. **CI Integration**: Proofs run on every PR
6. **New Method Policy**: Any new public method requires corresponding proof
7. **Modification Policy**: Changes to aggregate-affecting code require proof update

### Performance

8. **Individual Proof**: < 5 minutes each
9. **Full Suite**: < 30 minutes total
10. **Incremental**: Only re-run affected proofs on changes

## How This Prevents Future Bugs

| Scenario | Protection |
|----------|------------|
| Developer adds new PnL-modifying code | CI fails if no aggregate proof |
| Developer bypasses set_pnl() | aggregate_preserves proof fails |
| Developer forgets to update aggregate | computed vs stored mismatch |
| New operation introduced | Must add to proof matrix |

## Proof Matrix (Complete Coverage)

Every cell must have a proof:

| Operation | pnl_pos_tot | c_tot | conservation |
|-----------|-------------|-------|--------------|
| add_user | ✓ | ✓ | ✓ |
| add_lp | ✓ | ✓ | ✓ |
| deposit | ✓ | ✓ | ✓ |
| withdraw | ✓ | ✓ | ✓ |
| execute_trade | ✓ | ✓ | ✓ |
| settle_funding | ✓ | ✓ | ✓ |
| settle_warmup | ✓ | ✓ | ✓ |
| liquidate | ✓ | ✓ | ✓ |
| force_realize | ✓ | ✓ | ✓ |
| close_account | ✓ | ✓ | ✓ |
| gc_account | ✓ | ✓ | ✓ |
| **force_close** | ✓ | ✓ | ✓ |

When this matrix is complete, **Bug #10 class is proven impossible.**

## File Location

Add proofs to: `/home/anatoly/percolator/tests/kani.rs`

Section: `// PREMARKET RESOLUTION PROOFS`
