// lib.rs — Public re-export of both algorithms for Criterion benchmark harness.
// The canonical implementations live in main.rs; this shim exposes the types
// and functions to the benches/ crate without duplication.

/// Public module mirroring the two algorithms from main.rs with full visibility.
pub mod two_sets {

    // ── Domain Types ──────────────────────────────────────────────────────────

    /// Membership tag indicating which partition set an integer belongs to.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SetMembership {
        Alpha,
        Beta,
    }

    /// Feasibility verdict with an optional assignment vector.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PartitionVerdict {
        Infeasible,
        Feasible { assignment_vector: Vec<SetMembership> },
    }

    impl PartitionVerdict {
        #[inline]
        pub fn is_feasible(&self) -> bool {
            matches!(self, PartitionVerdict::Feasible { .. })
        }
    }

    // ── Feasibility Predicates ────────────────────────────────────────────────

    /// Returns true iff n % 4 ∈ {0, 3} — the balanced partition feasibility test.
    #[inline]
    pub fn is_partition_feasible(cardinality: u64) -> bool {
        matches!(cardinality % 4, 0 | 3)
    }

    /// Computes τ = n(n+1)/4 (exact when feasibility holds).
    #[inline]
    pub fn target_half_sum(cardinality: u64) -> u64 {
        cardinality * (cardinality + 1) / 4
    }

    // ── Algorithm A: Descending Greedy Scan Partitioner ───────────────────────

    /// Partitions {1..=n} via a greedy descending `.scan()` combinator pipeline.
    ///
    /// Processes n, n-1, ..., 1. State: `quota_remaining`.
    /// Claim element for Alpha if quota ≥ element; else Beta.
    /// Reverses collected vector to restore ascending index order.
    ///
    /// - Time:  O(n)
    /// - Space: O(n) for the assignment_vector
    pub fn descending_greedy_scan_partitioner(cardinality: u64) -> PartitionVerdict {
        if !is_partition_feasible(cardinality) {
            return PartitionVerdict::Infeasible;
        }

        let quota_threshold: u64 = target_half_sum(cardinality);

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

        assignment_vector.reverse();
        PartitionVerdict::Feasible { assignment_vector }
    }

    // ── Algorithm B: Two-Pointer Fold Partitioner ─────────────────────────────

    /// State vector for the two-pointer fold.
    #[derive(Debug, Clone)]
    pub struct TwoPointerState {
        pub low_pointer:       u64,
        pub high_pointer:      u64,
        pub quota_remaining:   u64,
        pub assignment_vector: Vec<SetMembership>,
    }

    impl TwoPointerState {
        pub fn identity(cardinality: u64, quota_threshold: u64) -> Self {
            Self {
                low_pointer:       1,
                high_pointer:      cardinality,
                quota_remaining:   quota_threshold,
                assignment_vector: vec![SetMembership::Beta; cardinality as usize],
            }
        }

        /// Processes two elements per call: high then low.
        pub fn advance(mut self) -> Self {
            if self.low_pointer > self.high_pointer {
                return self;
            }

            // High pointer
            let high_val = self.high_pointer;
            if self.quota_remaining >= high_val {
                self.quota_remaining -= high_val;
                self.assignment_vector[(high_val - 1) as usize] = SetMembership::Alpha;
            }
            self.high_pointer = self.high_pointer.saturating_sub(1);

            if self.low_pointer > self.high_pointer + 1 {
                return self;
            }

            // Low pointer
            let low_val = self.low_pointer;
            if self.quota_remaining >= low_val {
                self.quota_remaining -= low_val;
                self.assignment_vector[(low_val - 1) as usize] = SetMembership::Alpha;
            }
            self.low_pointer += 1;

            self
        }
    }

    /// Partitions {1..=n} via a two-pointer fold state-machine.
    ///
    /// Pre-allocates all elements as Beta, then alternately claims from
    /// high and low ends of [1, n] for Alpha in ⌈n/2⌉ advance() steps.
    ///
    /// - Time:  O(n)
    /// - Space: O(n) for the pre-allocated assignment_vector
    pub fn two_pointer_fold_partitioner(cardinality: u64) -> PartitionVerdict {
        if !is_partition_feasible(cardinality) {
            return PartitionVerdict::Infeasible;
        }

        let quota_threshold: u64 = target_half_sum(cardinality);
        let step_count: u64 = (cardinality + 1) / 2;

        let terminal_state: TwoPointerState =
            (0..step_count).fold(
                TwoPointerState::identity(cardinality, quota_threshold),
                |state, _| state.advance(),
            );

        PartitionVerdict::Feasible {
            assignment_vector: terminal_state.assignment_vector,
        }
    }

    // ── Benchmark Input Generators ────────────────────────────────────────────

    /// Nearest feasible n ≡ 0 (mod 4) ≤ scale.
    pub fn generate_feasible_n_mod0(scale: usize) -> u64 {
        let raw = scale as u64;
        (raw / 4) * 4
    }

    /// Nearest feasible n ≡ 3 (mod 4) ≤ scale.
    pub fn generate_feasible_n_mod3(scale: usize) -> u64 {
        let raw = scale as u64;
        let base = (raw / 4) * 4;
        if base + 3 <= raw { base + 3 } else { base.saturating_sub(1) }
    }

    /// Nearest infeasible n ≡ 1 (mod 4) ≤ scale.
    pub fn generate_infeasible_n(scale: usize) -> u64 {
        let raw = scale as u64;
        let base = (raw / 4) * 4;
        base + 1
    }
}
