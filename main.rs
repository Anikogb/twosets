// ============================================================================
// CSES Task 1092: Two Sets — Self-Contained Single-File Submission
// ============================================================================
// Problem:
//   Given n, determine whether the integers {1, 2, ..., n} can be divided
//   into two disjoint sets A and B such that sum(A) = sum(B).
//   If possible, print both sets; otherwise print "NO".
//
// Mathematical Core:
//   Let S = n(n+1)/2 (the triangular number T_n).
//   A partition into equal halves exists iff S is even, i.e. S ≡ 0 (mod 2).
//   Since S = n(n+1)/2, we have S even iff n(n+1) ≡ 0 (mod 4).
//   This holds precisely when n ≡ 0 (mod 4) or n ≡ 3 (mod 4).
//   Target half-sum: τ = S / 2 = n(n+1)/4.
//
// Key Insight — Why Descending Greedy Works:
//   Scanning 1..=n in descending order and greedily assigning each element
//   to Alpha while quota_remaining ≥ element always terminates with
//   quota_remaining = 0. Proof: the sequence of prefix sums of {n, n-1, ..., 1}
//   passes through every integer from n down to 0, so τ is always "hit exactly"
//   before the stream is exhausted.
//
// Two Algorithms (both O(n) time, O(n) space):
//
//   Algorithm A — DescendingGreedyScanPartitioner (Declarative Combinator):
//     Reverses the integer range n..=1 and applies a `.scan()` combinator that
//     threads a running `quota_remaining` as lazy state. Each element is claimed
//     for Set Alpha iff quota ≥ element; the scan terminates with an exhausted
//     quota. After collection, the assignment_vector is reversed to restore
//     ascending index order. Purely declarative; no explicit branching outside
//     the scan closure.
//
//   Algorithm B — AscendingTwoPointerFoldPartitioner (Functional State-Machine):
//     Uses two integer cursors — `low_pointer` (starting at 1) and
//     `high_pointer` (starting at n) — advancing them inward. At each fold
//     step, the cursor pair whose combined contribution brings the
//     accumulated `alpha_sum` closest to τ is assigned to Alpha; the other
//     pair element goes to Beta. The fold state encodes the full reduction
//     context as a named struct. This demonstrates a fundamentally different
//     constructive proof of the partition's existence.
//
// ============================================================================

use std::io::{self, Write};

// ── Shared Domain Types ───────────────────────────────────────────────────────

/// Membership tag indicating which partition set an integer belongs to.
///
/// Using an ADT (Algebraic Data Type) makes the partition semantics
/// explicit and pattern-matchable throughout every pipeline stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SetMembership {
    /// The element belongs to the first partition set (Set A).
    Alpha,
    /// The element belongs to the second partition set (Set B).
    Beta,
}

/// The total feasibility verdict for the partition problem.
///
/// This ADT eliminates boolean flags and makes the two outcomes
/// structurally distinct — impossible to confuse at the type level.
#[derive(Debug, Clone, PartialEq, Eq)]
enum PartitionVerdict {
    /// The partition is impossible for this value of n.
    Infeasible,
    /// A valid partition exists; carries the membership assignment vector.
    /// `assignment_vector[i]` is the `SetMembership` of integer `i + 1`.
    Feasible {
        assignment_vector: Vec<SetMembership>,
    },
}

impl PartitionVerdict {
    /// Returns `true` iff the verdict is `Feasible`.
    #[inline]
    #[allow(dead_code)]
    fn is_feasible(&self) -> bool {
        matches!(self, PartitionVerdict::Feasible { .. })
    }
}

// ── Feasibility Predicate ─────────────────────────────────────────────────────

/// Determines whether a balanced partition of {1..=n} is possible.
///
/// The triangular sum T_n = n(n+1)/2 must be even for an equal split.
/// Equivalently, n % 4 must be 0 or 3.
#[inline]
fn is_partition_feasible(cardinality: u64) -> bool {
    matches!(cardinality % 4, 0 | 3)
}

/// Computes the target half-sum τ = n(n+1)/4.
///
/// Caller must guarantee feasibility before invoking; the integer division
/// is exact under the feasibility condition.
#[inline]
fn target_half_sum(cardinality: u64) -> u64 {
    cardinality * (cardinality + 1) / 4
}

// ── Algorithm A: Descending Greedy Scan Partitioner ──────────────────────────

/// Partitions {1..=n} using a greedy descending-order combinator pipeline.
///
/// # Strategy
/// Reverses the integer range to process n, n-1, ..., 1 in descending order.
/// A `.scan()` combinator threads `quota_remaining` as lazy mutable state:
///   - If `quota_remaining ≥ integer_value`: assign Alpha, decrease quota by k.
///   - Otherwise: assign Beta (quota remains unchanged).
///
/// After collection, the assignment_vector is `.rev()`-restored to ascending
/// index order so `assignment_vector[i]` corresponds to integer `i + 1`.
///
/// # Why Descending?
/// Processing large elements first ensures the running quota passes through
/// τ exactly at some element boundary, yielding a perfect partition. Ascending
/// order cannot guarantee this — the cumulative prefix sums of {1,...,n} skip
/// large values and may overshoot τ without landing on it.
///
/// # Complexity
/// - Time:  O(n) — single linear scan over the integer range.
/// - Space: O(n) — the assignment_vector of length n.
///
/// # Arguments
/// * `cardinality` — the value n; caller must ensure `is_partition_feasible(n)`
fn descending_greedy_scan_partitioner(cardinality: u64) -> PartitionVerdict {
    if !is_partition_feasible(cardinality) {
        return PartitionVerdict::Infeasible;
    }

    let quota_threshold: u64 = target_half_sum(cardinality);

    // Scan descending: n, n-1, ..., 1.
    // State: quota_remaining. Once it hits 0, all remaining go to Beta.
    let mut assignment_vector: Vec<SetMembership> = (1u64..=cardinality)
        .rev()
        .scan(quota_threshold, |quota_remaining, integer_value| {
            if *quota_remaining >= integer_value {
                *quota_remaining -= integer_value;
                Some(SetMembership::Alpha)
            } else {
                Some(SetMembership::Beta)
            }
        })
        .collect();

    // Restore ascending index order: assignment_vector[0] ↔ integer 1.
    assignment_vector.reverse();

    PartitionVerdict::Feasible { assignment_vector }
}

// ── Algorithm B: Ascending Two-Pointer Fold Partitioner ──────────────────────

/// State vector for the two-pointer fold reduction.
///
/// Encodes the complete computation context as a named, immutable-transition
/// structure: two inward-moving cursors and a pre-allocated assignment buffer.
/// Both cursors advance every step — high from the right, low from the left.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct TwoPointerState {
    /// Left cursor: the smallest unassigned integer (starts at 1).
    low_pointer:       u64,
    /// Right cursor: the largest unassigned integer (starts at n).
    high_pointer:      u64,
    /// Remaining sum quota for Set Alpha.
    quota_remaining:   u64,
    /// Membership assignments (indexed 0..n; assignment_vector[i] = membership of i+1).
    assignment_vector: Vec<SetMembership>,
}

impl TwoPointerState {
    /// Constructs the initial (identity) fold state.
    fn identity(cardinality: u64, quota_threshold: u64) -> Self {
        Self {
            low_pointer:       1,
            high_pointer:      cardinality,
            quota_remaining:   quota_threshold,
            assignment_vector: vec![SetMembership::Beta; cardinality as usize],
        }
    }

    /// Advances one two-pointer step.
    ///
    /// # Protocol (each step processes TWO integers — one from each end):
    ///   1. Try to claim `high_pointer` for Alpha (fits quota → Alpha, advance right inward).
    ///   2. Try to claim `low_pointer` for Alpha (fits remaining quota → Alpha, advance left inward).
    ///   3. Both pointers advance regardless (unclaimed elements stay Beta).
    ///
    /// This alternating-pair strategy mirrors the mathematical proof that
    /// for n ≡ 0 (mod 4): pairs (n, 1), (n-1, 2), … each sum to n+1,
    /// and exactly n/4 such pairs form the target sum τ = n(n+1)/4.
    fn advance(mut self) -> Self {
        if self.low_pointer > self.high_pointer {
            return self;
        }

        // --- High pointer: greedy claim for Alpha ---
        let high_val = self.high_pointer;
        if self.quota_remaining >= high_val {
            self.quota_remaining -= high_val;
            self.assignment_vector[(high_val - 1) as usize] = SetMembership::Alpha;
        }
        // Advance high cursor inward unconditionally.
        self.high_pointer = self.high_pointer.saturating_sub(1);

        // Guard: if pointers crossed after advancing high, stop.
        if self.low_pointer > self.high_pointer + 1 {
            return self;
        }

        // --- Low pointer: greedy claim for Alpha ---
        let low_val = self.low_pointer;
        if self.quota_remaining >= low_val {
            self.quota_remaining -= low_val;
            self.assignment_vector[(low_val - 1) as usize] = SetMembership::Alpha;
        }
        // Advance low cursor inward unconditionally.
        self.low_pointer += 1;

        self
    }
}

/// Partitions {1..=n} using a functional two-pointer fold state-machine.
///
/// # Strategy
/// Pre-allocates all elements as Beta, then alternately claims elements from
/// the high and low ends of [1, n] for Alpha. Each `advance()` step processes
/// two integers (one from each end), giving O(n/2) total steps.
///
/// The fold is driven by a half-range iterator (0..n/2 steps), making the
/// control flow declarative at the call site — no explicit loop counters.
///
/// # Mathematical Basis
/// For n ≡ 0 (mod 4): pairs (n, 1), (n−1, 2), …, (n/2+1, n/2) each sum to
/// n+1, and exactly τ/(n+1) = n/4 such pairs form the target sum τ.
/// For n ≡ 3 (mod 4): element n alone anchors Alpha (since n ≡ 3, τ = n(n+1)/4
/// is reachable with n as the first claim), then pairs fill the rest.
///
/// # Complexity
/// - Time:  O(n) — ⌈n/2⌉ advance() calls, each O(1).
/// - Space: O(n) — the pre-allocated assignment_vector.
///
/// # Arguments
/// * `cardinality` — the value n; caller must ensure `is_partition_feasible(n)`
#[allow(dead_code)]
fn two_pointer_fold_partitioner(cardinality: u64) -> PartitionVerdict {
    if !is_partition_feasible(cardinality) {
        return PartitionVerdict::Infeasible;
    }

    let quota_threshold: u64 = target_half_sum(cardinality);
    // Each advance() step handles 2 elements; we need ⌈n/2⌉ steps.
    let step_count: u64 = (cardinality + 1) / 2;

    // Fold over half-range step indices, driving the two-pointer inward.
    let terminal_state: TwoPointerState =
        (0..step_count).fold(
            TwoPointerState::identity(cardinality, quota_threshold),
            |state, _step_index| state.advance(),
        );

    PartitionVerdict::Feasible {
        assignment_vector: terminal_state.assignment_vector,
    }
}

// ── Output Renderer ───────────────────────────────────────────────────────────

/// Renders a `PartitionVerdict` to the locked output stream.
///
/// Format (CSES-compliant):
///   - "NO\n" if Infeasible.
///   - "YES\n|A|\n a₁ a₂ …\n|B|\n b₁ b₂ …\n" if Feasible.
///
/// Uses a shared `BufWriter` to coalesce all writes into a single syscall flush.
fn render_verdict<W: Write>(
    verdict: &PartitionVerdict,
    output_stream: &mut W,
) -> io::Result<()> {
    match verdict {
        PartitionVerdict::Infeasible => {
            writeln!(output_stream, "NO")
        }
        PartitionVerdict::Feasible { assignment_vector } => {
            // Partition index list by membership using a fold into two accumulators.
            let (alpha_members, beta_members): (Vec<u64>, Vec<u64>) = assignment_vector
                .iter()
                .enumerate()
                .fold(
                    (Vec::new(), Vec::new()),
                    |(mut alpha_acc, mut beta_acc), (index, &membership)| {
                        let integer_value = index as u64 + 1;
                        match membership {
                            SetMembership::Alpha => alpha_acc.push(integer_value),
                            SetMembership::Beta  => beta_acc.push(integer_value),
                        }
                        (alpha_acc, beta_acc)
                    },
                );

            writeln!(output_stream, "YES")?;

            writeln!(output_stream, "{}", alpha_members.len())?;
            let alpha_line: String = alpha_members
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            writeln!(output_stream, "{alpha_line}")?;

            writeln!(output_stream, "{}", beta_members.len())?;
            let beta_line: String = beta_members
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            writeln!(output_stream, "{beta_line}")
        }
    }
}

// ── Entry Point ───────────────────────────────────────────────────────────────

fn main() -> io::Result<()> {
    let stdin_handle  = io::stdin();
    let stdout_handle = io::stdout();

    let mut header_buffer = String::new();
    {
        use std::io::BufRead;
        let mut input_stream = stdin_handle.lock();
        input_stream.read_line(&mut header_buffer)?;
    }
    let cardinality: u64 = header_buffer
        .trim()
        .parse()
        .expect("invalid cardinality token in input stream");

    // Primary submission path: Algorithm A (DescendingGreedyScanPartitioner).
    // Algorithm B (TwoPointerFoldPartitioner) is available in lib.rs for
    // benchmarking and equivalence verification.
    let verdict = descending_greedy_scan_partitioner(cardinality);

    let mut output_stream = io::BufWriter::new(stdout_handle.lock());
    render_verdict(&verdict, &mut output_stream)?;
    output_stream.flush()
}

// ── Test Suite ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helpers ──────────────────────────────────────────────────────────────

    /// Verifies that an assignment_vector produces two equal-sum sets
    /// and covers every integer in {1..=cardinality} exactly once.
    fn verify_valid_partition(cardinality: u64, assignment_vector: &[SetMembership]) {
        assert_eq!(
            assignment_vector.len(),
            cardinality as usize,
            "assignment_vector length mismatch for n = {}",
            cardinality
        );

        let (sum_alpha, sum_beta): (u64, u64) = assignment_vector
            .iter()
            .enumerate()
            .fold((0u64, 0u64), |(acc_alpha, acc_beta), (index, &membership)| {
                let integer_value = index as u64 + 1;
                match membership {
                    SetMembership::Alpha => (acc_alpha + integer_value, acc_beta),
                    SetMembership::Beta  => (acc_alpha, acc_beta + integer_value),
                }
            });

        assert_eq!(
            sum_alpha, sum_beta,
            "Unequal partition sums for n = {}: α={}, β={}",
            cardinality, sum_alpha, sum_beta
        );
    }

    /// Runs both algorithms on `cardinality` and asserts cross-algorithm agreement.
    fn assert_both_algorithms_agree(cardinality: u64) {
        let verdict_a = descending_greedy_scan_partitioner(cardinality);
        let verdict_b = two_pointer_fold_partitioner(cardinality);

        assert_eq!(
            verdict_a.is_feasible(),
            verdict_b.is_feasible(),
            "Feasibility divergence for n = {}",
            cardinality
        );

        if let PartitionVerdict::Feasible { assignment_vector: av_a } = &verdict_a {
            verify_valid_partition(cardinality, av_a);
        }
        if let PartitionVerdict::Feasible { assignment_vector: av_b } = &verdict_b {
            verify_valid_partition(cardinality, av_b);
        }
    }

    // ── Feasibility Predicate Tests ───────────────────────────────────────────

    /// n = 1: T_1 = 1, odd → Infeasible.
    #[test]
    fn test_feasibility_n1_infeasible() {
        assert!(!is_partition_feasible(1));
        assert_eq!(descending_greedy_scan_partitioner(1), PartitionVerdict::Infeasible);
        assert_eq!(two_pointer_fold_partitioner(1),       PartitionVerdict::Infeasible);
    }

    /// n = 2: T_2 = 3, odd → Infeasible.
    #[test]
    fn test_feasibility_n2_infeasible() {
        assert!(!is_partition_feasible(2));
        assert_eq!(descending_greedy_scan_partitioner(2), PartitionVerdict::Infeasible);
    }

    /// n = 3: T_3 = 6, even → Feasible. τ = 3. {3} vs {1,2}.
    #[test]
    fn test_feasibility_n3_feasible() {
        assert!(is_partition_feasible(3));
        assert_both_algorithms_agree(3);
    }

    /// n = 4: T_4 = 10, even → Feasible. τ = 5. {4,1} vs {3,2}.
    #[test]
    fn test_feasibility_n4_feasible() {
        assert!(is_partition_feasible(4));
        assert_both_algorithms_agree(4);
    }

    /// n = 5: T_5 = 15, odd → Infeasible.
    #[test]
    fn test_feasibility_n5_infeasible() {
        assert!(!is_partition_feasible(5));
        assert_eq!(descending_greedy_scan_partitioner(5), PartitionVerdict::Infeasible);
    }

    /// n = 6: T_6 = 21, odd → Infeasible.
    #[test]
    fn test_feasibility_n6_infeasible() {
        assert!(!is_partition_feasible(6));
    }

    /// n = 7: T_7 = 28, even → Feasible. τ = 14.
    #[test]
    fn test_feasibility_n7_feasible() {
        assert!(is_partition_feasible(7));
        assert_both_algorithms_agree(7);
    }

    /// n = 8: T_8 = 36, even → Feasible. τ = 18.
    #[test]
    fn test_feasibility_n8_feasible() {
        assert!(is_partition_feasible(8));
        assert_both_algorithms_agree(8);
    }

    // ── Correctness: Residue Class Sweep (n = 1..=20) ────────────────────────

    /// Sweep n = 1..=20: verify feasibility matches n%4∈{0,3}
    /// and every feasible case produces a valid equal-sum partition.
    #[test]
    fn test_residue_class_sweep_n1_to_20() {
        for cardinality in 1u64..=20 {
            let expected_feasible = matches!(cardinality % 4, 0 | 3);
            assert_eq!(
                is_partition_feasible(cardinality),
                expected_feasible,
                "Feasibility predicate mismatch for n = {}",
                cardinality
            );
            assert_both_algorithms_agree(cardinality);
        }
    }

    // ── Correctness: Exact Sum Verification ──────────────────────────────────

    /// n = 3: τ = 3. Expected alpha-sum == beta-sum == 3.
    #[test]
    fn test_exact_partition_n3() {
        let verdict = descending_greedy_scan_partitioner(3);
        if let PartitionVerdict::Feasible { assignment_vector } = &verdict {
            verify_valid_partition(3, assignment_vector);
        } else {
            panic!("Expected Feasible for n = 3");
        }
    }

    /// n = 4: τ = 5. Expected alpha-sum == beta-sum == 5.
    #[test]
    fn test_exact_partition_n4() {
        assert_both_algorithms_agree(4);
    }

    /// n = 7: τ = 14. Expected alpha-sum == beta-sum == 14.
    #[test]
    fn test_exact_partition_n7() {
        assert_both_algorithms_agree(7);
    }

    /// n = 8: τ = 18. Expected alpha-sum == beta-sum == 18.
    #[test]
    fn test_exact_partition_n8() {
        assert_both_algorithms_agree(8);
    }

    // ── Boundary / Maximum Constraint Cases ──────────────────────────────────

    /// Minimum infeasible case.
    #[test]
    fn test_minimum_infeasible_n1() {
        assert_eq!(descending_greedy_scan_partitioner(1), PartitionVerdict::Infeasible);
    }

    /// Minimum feasible case.
    #[test]
    fn test_minimum_feasible_n3() {
        assert_both_algorithms_agree(3);
    }

    /// Large feasible case: n = 999_999 ≡ 3 (mod 4). T ≡ 0 (mod 2). ✓
    #[test]
    fn test_large_n_congruent_3_mod4() {
        let cardinality: u64 = 999_999;
        assert_eq!(cardinality % 4, 3);
        assert!(is_partition_feasible(cardinality));
        assert_both_algorithms_agree(cardinality);
    }

    /// Large feasible case: n = 1_000_000 ≡ 0 (mod 4). T ≡ 0 (mod 2). ✓
    #[test]
    fn test_large_n_congruent_0_mod4() {
        let cardinality: u64 = 1_000_000;
        assert_eq!(cardinality % 4, 0);
        assert!(is_partition_feasible(cardinality));
        assert_both_algorithms_agree(cardinality);
    }

    /// Large infeasible case: n = 999_998 ≡ 2 (mod 4). T is odd. ✗
    #[test]
    fn test_large_n_infeasible_congruent_2_mod4() {
        let cardinality: u64 = 999_998;
        assert_eq!(cardinality % 4, 2);
        assert!(!is_partition_feasible(cardinality));
        assert_eq!(descending_greedy_scan_partitioner(cardinality), PartitionVerdict::Infeasible);
    }

    // ── Algorithm Equivalence Property Tests ─────────────────────────────────

    /// Exhaustive equivalence check: both algorithms must agree on n = 1..=100.
    #[test]
    fn test_algorithm_equivalence_sweep_n1_to_100() {
        for cardinality in 1u64..=100 {
            assert_both_algorithms_agree(cardinality);
        }
    }

    /// Stress test: n = 1_000_000 (maximum constraint).
    /// Both algorithms must return Feasible with equal-sum partitions.
    #[test]
    fn test_stress_maximum_n() {
        assert_both_algorithms_agree(1_000_000);
    }

    // ── PartitionVerdict ADT Structural Tests ─────────────────────────────────

    /// Verify `is_feasible()` on both variants.
    #[test]
    fn test_partition_verdict_adt_properties() {
        assert!(!PartitionVerdict::Infeasible.is_feasible());
        assert!(PartitionVerdict::Feasible {
            assignment_vector: vec![SetMembership::Alpha, SetMembership::Beta]
        }
        .is_feasible());
    }

    /// Verify target_half_sum for known values.
    #[test]
    fn test_target_half_sum_known_values() {
        // n = 3: S = 6, τ = 3
        assert_eq!(target_half_sum(3), 3);
        // n = 4: S = 10, τ = 5
        assert_eq!(target_half_sum(4), 5);
        // n = 7: S = 28, τ = 14
        assert_eq!(target_half_sum(7), 14);
        // n = 8: S = 36, τ = 18
        assert_eq!(target_half_sum(8), 18);
        // n = 1_000_000: S = 1_000_000 * 1_000_001 / 2 = 500_000_500_000, τ = 250_000_250_000
        assert_eq!(target_half_sum(1_000_000), 250_000_250_000);
    }

    /// Validates TwoPointerState transition invariant for n = 4 manually.
    ///
    /// n = 4, τ = 5. advance() processes TWO elements per call:
    ///   Step 1: high=4 ≤ quota=5 → Alpha (quota→1); high→3.
    ///           low=1  ≤ quota=1 → Alpha (quota→0); low→2.
    ///   Step 2: high=3 > quota=0 → Beta; high→2.
    ///           low=2  > quota=0 → Beta; low→3.
    ///   Final:  {1,4}=Alpha, {2,3}=Beta. Sums: 5 == 5. ✓
    #[test]
    fn test_two_pointer_state_transition_invariant_n4() {
        let state_0 = TwoPointerState::identity(4, 5);
        assert_eq!(state_0.quota_remaining, 5);
        assert_eq!(state_0.low_pointer,     1);
        assert_eq!(state_0.high_pointer,    4);

        // Single advance() call processes integers 4 (high) and 1 (low).
        let state_1 = state_0.advance();
        assert_eq!(state_1.quota_remaining, 0);
        assert_eq!(state_1.assignment_vector[3], SetMembership::Alpha); // integer 4
        assert_eq!(state_1.assignment_vector[0], SetMembership::Alpha); // integer 1

        // Full verdict must also be correct.
        let verdict = two_pointer_fold_partitioner(4);
        if let PartitionVerdict::Feasible { assignment_vector } = &verdict {
            verify_valid_partition(4, assignment_vector);
        } else {
            panic!("Expected Feasible for n = 4");
        }
    }
}

