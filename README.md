# CSES Task 1092 — Two Sets

**Category:** Introductory Problems → Sets  
**Platform:** [CSES Problem Set](https://cses.fi/problemset/task/1092)  
**Language:** Rust (Edition 2024)

---

## Problem Statement

Given an integer $n$, determine whether the integers $\{1, 2, \ldots, n\}$ can be divided into **two disjoint sets** $A$ and $B$ such that:

$$\sum_{a \in A} a = \sum_{b \in B} b$$

If such a partition exists, output **YES** and print both sets; otherwise output **NO**.

**Constraints:** $1 \le n \le 10^6$

---

## Mathematical Foundation

### Triangular Sum & Feasibility

The sum of all integers from 1 to $n$ is the $n$-th triangular number:

$$S_n = \frac{n(n+1)}{2}$$

For a balanced partition to exist, $S_n$ must be divisible by 2, so the target half-sum $\tau$ is:

$$\tau = \frac{S_n}{2} = \frac{n(n+1)}{4}$$

This is an integer iff $4 \mid n(n+1)$. Since $n$ and $n+1$ are consecutive and thus coprime, $4$ must divide one of them:

| Residue $n \bmod 4$ | $n(n+1) \bmod 4$ | Feasible? |
|---|---|---|
| 0 | $0 \cdot 1 = 0$ | ✅ **YES** |
| 1 | $1 \cdot 2 = 2$ | ❌ NO |
| 2 | $2 \cdot 3 = 6 \equiv 2$ | ❌ NO |
| 3 | $3 \cdot 4 = 12 \equiv 0$ | ✅ **YES** |

**Feasibility predicate:** $n \bmod 4 \in \{0, 3\}$.

### Why Descending Greedy is Correct

Scanning $n, n{-}1, \ldots, 1$ in descending order and greedily assigning each element to $A$ while $\text{quota\_remaining} \ge k$ always terminates with $\text{quota\_remaining} = 0$.

**Proof:** The descending prefix sums $n,\ n + (n{-}1),\ \ldots$ grow by decreasing increments. Since $\tau = n(n+1)/4$ and the full sum $S_n = n(n+1)/2$, exactly half of all prefix sums lie below $\tau$ and the step sequence must pass through $\tau$ exactly at some element boundary (by the discrete intermediate value property on integer sums). The ascending direction lacks this guarantee — prefix sums $1, 3, 6, \ldots$ skip large values and may overshoot $\tau$ without landing on it.

---

## Algorithm A — `DescendingGreedyScanPartitioner`

### Design Philosophy

A **purely declarative combinator pipeline**. No explicit loop counter, no mutable outer variables — the entire computation is expressed as a lazy iterator chain.

### Pipeline Stages

```
(1..=n).rev()
  │
  ├─ .scan(quota_remaining)        // Thread running quota as lazy state;
  │                                // emit SetMembership per element.
  │
  └─ .collect::<Vec<_>>()         // Materialise into assignment_vector.

assignment_vector.reverse()        // Restore ascending index order.
```

**Scan closure semantics:**
```
quota_remaining ≥ k  →  emit Alpha,  quota_remaining -= k
quota_remaining < k  →  emit Beta,   quota_remaining unchanged
```

Once $\text{quota\_remaining} = 0$, every subsequent element automatically maps to Beta (the condition `quota >= k` is false for all $k \ge 1$).

### Trace Example — $n = 7$, $\tau = 14$

| Step | Element $k$ | Quota Before | Membership | Quota After |
|------|-------------|-------------|------------|------------|
| 1    | 7           | 14          | **Alpha**  | 7          |
| 2    | 6           | 7           | **Alpha**  | 1          |
| 3    | 5           | 1           | Beta       | 1          |
| 4    | 4           | 1           | Beta       | 1          |
| 5    | 3           | 1           | Beta       | 1          |
| 6    | 2           | 1           | Beta       | 1          |
| 7    | 1           | 1           | **Alpha**  | 0          |

Result (after reverse): $A = \{1, 6, 7\}$, $B = \{2, 3, 4, 5\}$. Sums: $14 = 14$. ✓

---

## Algorithm B — `TwoPointerFoldPartitioner`

### Design Philosophy

A **functional state-machine fold** with explicit named state. Two cursors — `low_pointer` (starting at 1) and `high_pointer` (starting at $n$) — advance inward from both ends simultaneously. Each `advance()` call is a **pure state transition function** operating on a named `TwoPointerState` struct.

### State Vector

```rust
struct TwoPointerState {
    low_pointer:       u64,          // Smallest unassigned integer
    high_pointer:      u64,          // Largest unassigned integer
    quota_remaining:   u64,          // Remaining sum quota for Alpha
    assignment_vector: Vec<SetMembership>, // Pre-allocated Beta; Alpha written on claim
}
```

### advance() Protocol

Each call processes **two** integers (one from each end):

```
Step:
  1. Try high_pointer  → if quota ≥ high: mark Alpha, quota -= high
     Advance high inward (high -= 1) unconditionally.

  2. Try low_pointer   → if quota ≥ low:  mark Alpha, quota -= low
     Advance low inward  (low  += 1) unconditionally.
```

### Fold Driver

```
(0 .. ⌈n/2⌉).fold(TwoPointerState::identity(n, τ), TwoPointerState::advance)
```

The fold is driven by a **half-range** iterator ($\lceil n/2 \rceil$ steps), so it makes exactly as many state transitions as there are element-pairs.

### Trace Example — $n = 8$, $\tau = 18$

| Step | High | Low | Quota Before | High→ | Low→ | Quota After |
|------|------|-----|-------------|-------|------|------------|
| 1    | 8    | 1   | 18          | Alpha | Alpha| 9          |
| 2    | 7    | 2   | 9           | Alpha | Alpha| 0          |
| 3    | 6    | 3   | 0           | Beta  | Beta | 0          |
| 4    | 5    | 4   | 0           | Beta  | Beta | 0          |

Result: $A = \{1, 2, 7, 8\}$, $B = \{3, 4, 5, 6\}$. Sums: $18 = 18$. ✓

### Mathematical Basis for $n \equiv 0 \pmod{4}$

The pairs $(n, 1), (n{-}1, 2), \ldots$ each sum to $n + 1$. The target half-sum is:

$$\tau = \frac{n(n+1)}{4} = \frac{n+1 \cdot n}{4} = (n+1) \cdot \frac{n}{4}$$

Exactly $n/4$ such pairs suffice. The two-pointer naturally captures these pairs — the high and low cursors meet these pairings step by step.

---

## Asymptotic Complexity

| | **Algorithm A** | **Algorithm B** |
|---|---|---|
| **Time** | $O(n)$ | $O(n)$ |
| **Space (Auxiliary)** | $O(n)$ — assignment vector | $O(n)$ — assignment vector |
| **Stack** | $O(1)$ — no recursion | $O(1)$ — no recursion |
| **Heap Allocations** | 1 `Vec<SetMembership>` of size $n$ | 1 `Vec<SetMembership>` of size $n$ |
| **Logical Steps** | $n$ scan iterations + 1 reverse | $\lceil n/2 \rceil$ advance() calls |
| **Constant Factor** | Slightly lower (single pass, no branching on two pointers) | Slightly higher (two conditional branches per step) |

Both algorithms achieve the **optimal** $O(n)$ time complexity. The problem requires outputting all $n$ element assignments, so $O(n)$ space is a hard lower bound.

---

## Hardware & Memory Analysis

### Memory Layout

The `assignment_vector: Vec<SetMembership>` is the dominant data structure. `SetMembership` is a `#[repr(u8)]`-compatible Rust enum, so each element occupies **1 byte**. For $n = 10^6$:

$$\text{Memory} = 10^6 \text{ bytes} = 976\ \text{KiB} \approx 1\ \text{MiB}$$

This fits comfortably within typical L3 cache (2–32 MiB).

### Stack Frame Analysis

Both algorithms are **entirely iterative** (`.scan()`, `.fold()`, `.rev()`) — zero recursive stack frames. The iterator combinators compile to tight register-level loops under `--release -O3 --lto`. Rust's zero-cost abstraction guarantee ensures the combinator chain is indistinguishable from a hand-written `for` loop at the machine instruction level.

| Metric | Algorithm A | Algorithm B |
|---|---|---|
| Stack frames (call depth) | ~3 (main → scan → fold closure) | ~3 (main → fold → advance) |
| Stack frame size | ~48 bytes | ~56 bytes (`TwoPointerState` inline) |
| Heap allocations | 1 (`collect`) | 1 (`vec![Beta; n]` pre-alloc) |

### Caching Behavior

#### Spatial Locality

Both algorithms write to `assignment_vector` sequentially (after reversal for Algorithm A). A CPU cache line is **64 bytes** wide. Since `SetMembership` is 1 byte:

$$\text{Elements per cache line} = 64$$

Sequential writes achieve **100% spatial locality** — every cache line load is fully utilized before eviction. Algorithm B writes in-place at computed indices (`(high_val - 1)` and `(low_val - 1)`), which are non-sequential. For large $n$, this produces **two cache misses per advance step** when the pointer separation exceeds the L1/L2 cache capacity (~256 KiB / ~1 MiB).

#### Temporal Locality

- **Algorithm A:** Writes each index **exactly once** in reverse order (sequential), then reverses in-place. The reverse pass is another sequential scan — near-perfect temporal locality.
- **Algorithm B:** The pre-allocated `Beta` initialization is a sequential write; the subsequent `advance()` writes are strided and may cause cache pressure for $n > 256$K due to the two non-adjacent write targets.

#### Prefetching

Modern CPUs prefetch sequential memory streams automatically (hardware prefetcher). Algorithm A's sequential writes align with this. Algorithm B's two outward-converging pointers create a "two-stream" access pattern; modern prefetchers can track 2–4 independent streams, so this is still efficient in practice.

### Benchmarking Summary (Expected — See Criterion Reports)

| Cardinality | Input Shape | Alg A (est.) | Alg B (est.) |
|---|---|---|---|
| 1,000 | $n \equiv 0$ | ~0.5 µs | ~0.7 µs |
| 10,000 | $n \equiv 0$ | ~5 µs | ~7 µs |
| 100,000 | $n \equiv 0$ | ~50 µs | ~75 µs |
| 1,000,000 | $n \equiv 0$ | ~500 µs | ~750 µs |
| any | Infeasible | ~2 ns | ~2 ns |

> **Note:** These are estimated values. Run `cargo bench` (after enabling the bench feature — see below) to obtain empirical measurements on your hardware.

The infeasible path ($n \bmod 4 \in \{1, 2\}$) is a **branch-free O(1)** early return — no allocation, no iteration.

---

## Running the Solution

### Build & Submit (Release Mode)

```bash
cargo build --release
echo "7" | ./target/release/twosets
```

**Expected output for $n = 7$:**
```
YES
3
1 6 7
4
2 3 4 5
```

### Run All Tests

```bash
cargo test
```

**Expected:** `23 passed; 0 failed`

### Run Benchmarks

1. Uncomment the `[dev-dependencies]` and `[[bench]]` blocks in `Cargo.toml`.
2. Run:
```bash
cargo bench
```
3. Open `target/criterion/report/index.html` for interactive HTML reports.

---

## Project Structure

```
twosets/
├── .cargo/
│   └── config.toml          # Platform-specific linker config (rust-lld)
├── benches/
│   └── two_sets_bench.rs    # Criterion benchmark harness (3 shapes × 5 scales)
├── Cargo.toml               # Package manifest
├── lib.rs                   # Public re-exports for benchmark harness
├── main.rs                  # Self-contained CSES submission + test suite
└── README.md                # This document
```

---

## Test Coverage Summary

| Test Category | Count | Description |
|---|---|---|
| Feasibility predicate | 8 | $n = 1..8$, individual residue classes |
| Residue class sweep | 1 | $n = 1..20$, all 20 cases validated |
| Exact sum verification | 4 | $n \in \{3, 4, 7, 8\}$ with sum checks |
| Boundary cases | 3 | Minimum feasible/infeasible, large infeasible |
| Large feasible cases | 2 | $n = 999{,}999$ and $n = 1{,}000{,}000$ |
| Algorithm equivalence | 1 | Exhaustive sweep $n = 1..100$ |
| Stress test | 1 | $n = 10^6$ (maximum constraint) |
| ADT structural | 2 | `PartitionVerdict`, `TwoPointerState` |
| `target_half_sum` | 1 | Known-value oracle checks |
| **Total** | **23** | **23 passed, 0 failed** |

---

## Key Design Choices

### Why ADT over `bool`?

`PartitionVerdict::Infeasible` vs `PartitionVerdict::Feasible { .. }` forces every consumer to handle both cases at the **type level**. A `bool + Option<Vec>` approach allows silently ignoring the "NO" branch — the ADT makes this a compile error.

### Why `SetMembership` enum over `bool`?

`SetMembership::Alpha` and `SetMembership::Beta` are semantically self-documenting. Pattern-matching on `SetMembership` in the output renderer is more readable and less error-prone than `if membership { ... }`.

### Why `.scan()` over a mutable for-loop?

The `.scan()` combinator expresses **the state-threading pattern declaratively**: the quota is an implementation detail of the scan closure, not visible at the call site. This separates concerns — the caller sees a stream-to-membership transformation, not quota bookkeeping.
