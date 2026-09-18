//! starforge-math — pure port of STARFORGE `math-spec.json` / `StarforgeGame.sol`.
//!
//! Grid layout: 6 cols x 5 rows, **column-major** index = col*5 + row
//! (matches the Solidity contract). No wild symbol.
//! Symbols: 0..6 minerals (copper, iron, nickel, silver, gold, platinum,
//! neutronium), 7 = star (scatter).
//!
//! All payouts are in **bps of wager** (10000 = 1x). No Solana/Anchor
//! dependencies: this crate is also the reference for parity tests.

use sha2::{Digest, Sha256};

fn sha256_hash(input: &[u8; 32]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(input);
    h.finalize().into()
}

#[cfg(feature = "parity-keccak")]
fn keccak256_hash(input: &[u8; 32]) -> [u8; 32] {
    use tiny_keccak::{Hasher, Keccak};
    let mut k = Keccak::v256();
    let mut out = [0u8; 32];
    k.update(input);
    k.finalize(&mut out);
    out
}

// ---------------- constants (mirror math-spec.json) ----------------

pub const COLS: usize = 6;
pub const ROWS: usize = 5;
pub const CELLS: usize = 30;

pub const STAR: u8 = 7;

/// Base weights per cell: copper30 iron26 nickel22 silver16 gold12 platinum8 neutronium5 star3
pub const WEIGHTS: [u16; 8] = [30, 26, 22, 16, 12, 8, 5, 3];
pub const TOTAL_WEIGHT: u16 = 122;
/// uint16 rejection sampling: floor(65536/122)*122 = 65514
pub const SYMBOL_REJECT: u16 = 65514;

/// Scatter paytables, bps of wager, tiers [8-9, 10-11, 12+], indexed by mineral 0..6.
pub const PAY_8: [u32; 7] = [1650, 2475, 4125, 6600, 8250, 16500, 41250];
pub const PAY_10: [u32; 7] = [4125, 6600, 8250, 12375, 20625, 41250, 99000];
pub const PAY_12: [u32; 7] = [16500, 24750, 33000, 49500, 82500, 165000, 412500];

/// Constellation patterns: (pay_bps, cells col*5+row). Checked highest-pay first.
pub const PATTERNS: [(u32, [u8; 5]); 4] = [
    (20_000, [11, 12, 13, 7, 17]),   // cruz 2x
    (40_000, [6, 18, 12, 8, 16]),    // equis 4x
    (80_000, [10, 14, 2, 22, 12]),   // diamante 8x
    (150_000, [6, 7, 8, 13, 18]),    // herradura 15x
];

/// Supernova fixed prize multiset (xbet bps), shuffled by RNG.
pub const PRIZES: [u16; 12] = [
    10000, 10000, 10000, 20000, 20000, 20000, 30000, 30000, 40000, 50000, 60000, 60000,
];
pub const SN_TRIGGER: u8 = 4;
pub const SN_PICKS: usize = 5;

/// Caps & risk
pub const MAX_PAYOUT_X: u64 = 1000;
/// Declared RTP, bps (steady-state 10M Monte Carlo, see docs/RTP_MATH.md)
pub const RTP_BPS: u64 = 9760;

/// Artifact bitmask bits
pub const ART_BRASA: u8 = 0x01; // constellation pattern pays x1.25
pub const ART_YUNQUE: u8 = 0x02; // +5% on tier-3 (12+) scatter pays
pub const ART_TEMPLE: u8 = 0x04; // supernova pick prizes +10%

pub const CASCADE_CAP: u8 = 2;

pub const STAGE_SPIN: u8 = 0;
pub const STAGE_PICKS: u8 = 1;
pub const STAGE_GAMBLE: u8 = 2;
pub const STAGE_DONE: u8 = 3;

// ---------------- RNG (rejection sampling over a hash cursor) ----------------

/// Byte cursor over keccak-style chained hashing. The Solidity contract uses
/// keccak256; Solana programs use sha256. **Parity note:** the on-chain seed
/// pipeline differs (sha256 vs keccak), but the *sampling algorithm* (rejection
/// limits, draw order) is identical, so statistical properties and RTP match.
/// Byte-level parity tests target the algorithm, not the hash function.
pub struct Cursor {
    seed: [u8; 32],
    idx: usize,
    hash: fn(&[u8; 32]) -> [u8; 32],
}

impl Cursor {
    /// Solana-native cursor: sha256-chained. Used on-chain.
    pub fn new(seed: [u8; 32]) -> Self {
        Self {
            seed,
            idx: 0,
            hash: sha256_hash,
        }
    }

    /// Keccak-chained cursor: ONLY for parity against the Python/EVM reference.
    /// Byte-identical to `simulator/parity.py`'s Cursor. Never used on-chain.
    #[cfg(feature = "parity-keccak")]
    pub fn new_keccak(seed: [u8; 32]) -> Self {
        Self {
            seed,
            idx: 0,
            hash: keccak256_hash,
        }
    }

    pub fn next_byte(&mut self) -> u8 {
        if self.idx >= 32 {
            self.seed = (self.hash)(&self.seed);
            self.idx = 0;
        }
        let b = self.seed[self.idx];
        self.idx += 1;
        b
    }

    /// uniform symbol in [0, TOTAL_WEIGHT): reject v >= SYMBOL_REJECT
    pub fn draw_symbol(&mut self) -> u8 {
        loop {
            let v = ((self.next_byte() as u16) << 8) | self.next_byte() as u16;
            if v < SYMBOL_REJECT {
                let w = v % TOTAL_WEIGHT;
                if w < 30 { return 0; }
                if w < 56 { return 1; }
                if w < 78 { return 2; }
                if w < 94 { return 3; }
                if w < 106 { return 4; }
                if w < 114 { return 5; }
                if w < 119 { return 6; }
                return STAR;
            }
        }
    }

    /// uniform u8 in [0, n): limit = floor(256/n)*n, reject b >= limit
    pub fn draw_range(&mut self, n: u8) -> u8 {
        debug_assert!(n > 0);
        let limit = (256u16 / n as u16) * n as u16;
        loop {
            let b = self.next_byte();
            if (b as u16) < limit {
                return b % n;
            }
        }
    }
}

// ---------------- paytable ----------------

pub fn pay_for(mineral: u8, count: u8) -> u32 {
    let table = if count >= 12 {
        &PAY_12
    } else if count >= 10 {
        &PAY_10
    } else if count >= 8 {
        &PAY_8
    } else {
        return 0;
    };
    table[mineral as usize]
}

fn match_pattern(grid: &[u8; CELLS], cells: &[u8; 5]) -> bool {
    let first = grid[cells[0] as usize];
    if first > 6 {
        return false; // stars can't form a pattern
    }
    cells[1..].iter().all(|&c| grid[c as usize] == first)
}

// ---------------- spin ----------------

/// Result of one cascade step evaluation.
struct StepEval {
    win_bps: u64,
    pat_id: u8, // 0=none, 1=cruz, 2=equis, 3=diamante, 4=herradura
    pay_sym: [bool; 9],
    any_pay: bool,
}

fn evaluate_step(grid: &[u8; CELLS], artifacts: u8) -> StepEval {
    let mut counts = [0u8; 8];
    for &s in grid.iter() {
        counts[s as usize] += 1;
    }
    let mut scatter_bps: u64 = 0;
    let mut tier3_bps: u64 = 0;
    let mut pay_sym = [false; 9];
    for m in 0..7u8 {
        let p = pay_for(m, counts[m as usize]);
        if p > 0 {
            scatter_bps += p as u64;
            if counts[m as usize] >= 12 {
                tier3_bps += p as u64;
            }
            pay_sym[m as usize] = true;
        }
    }
    // yunque: +5% on tier-3 (12+) scatter pays only
    if artifacts & ART_YUNQUE != 0 {
        scatter_bps += tier3_bps / 20;
    }

    // best pattern wins (checked highest-pay first)
    let mut pat_id = 0u8;
    let mut pat_bps: u64 = 0;
    for (i, (pay, cells)) in PATTERNS.iter().enumerate() {
        if match_pattern(grid, cells) && *pay as u64 > pat_bps {
            pat_id = (i + 1) as u8;
            pat_bps = *pay as u64;
        }
    }
    // brasa: constellation pays x1.25
    if artifacts & ART_BRASA != 0 {
        pat_bps = pat_bps * 5 / 4;
    }

    StepEval {
        win_bps: scatter_bps + pat_bps,
        pat_id,
        pay_sym,
        any_pay: scatter_bps > 0,
    }
}

fn collapse_and_refill(grid: &mut [u8; CELLS], pay_sym: &[bool; 9], cur: &mut Cursor) {
    for c in 0..COLS {
        let mut surv = [0u8; ROWS];
        let mut n = 0usize;
        for r in 0..ROWS {
            let s = grid[c * ROWS + r];
            if !pay_sym[s as usize] {
                surv[n] = s;
                n += 1;
            }
        }
        let top = ROWS - n;
        for r in 0..ROWS {
            grid[c * ROWS + r] = if r < top { cur.draw_symbol() } else { surv[r - top] };
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpinOutcome {
    pub grids: Vec<[u8; CELLS]>, // pre-refill grid per cascade step
    pub step_wins_bps: Vec<u64>,
    pub step_pats: Vec<u8>,
    pub grid_win_bps: u64,
    pub stars: u8,
    pub prizes: Option<[u16; 12]>, // Some iff supernova triggered (shuffled)
}

/// Full spin: draws, cascades (cap 2), patterns, supernova check + prize shuffle.
/// Uses the Solana-native sha256 cursor (on-chain path).
pub fn run_spin(seed: [u8; 32], artifacts: u8) -> SpinOutcome {
    run_spin_cursor(Cursor::new(seed), artifacts)
}

/// Full spin with the keccak-chained cursor — parity against the Python/EVM
/// reference ONLY. Requires the `parity-keccak` feature; never used on-chain.
#[cfg(feature = "parity-keccak")]
pub fn run_spin_keccak(seed: [u8; 32], artifacts: u8) -> SpinOutcome {
    run_spin_cursor(Cursor::new_keccak(seed), artifacts)
}

fn run_spin_cursor(mut cur: Cursor, artifacts: u8) -> SpinOutcome {
    let mut grid = [0u8; CELLS];
    for i in 0..CELLS {
        grid[i] = cur.draw_symbol();
    }
    let stars = grid.iter().filter(|&&s| s == STAR).count() as u8;

    let mut out = SpinOutcome {
        grids: Vec::new(),
        step_wins_bps: Vec::new(),
        step_pats: Vec::new(),
        grid_win_bps: 0,
        stars,
        prizes: None,
    };

    for _ in 0..CASCADE_CAP {
        let ev = evaluate_step(&grid, artifacts);
        out.grids.push(grid);
        out.step_wins_bps.push(ev.win_bps);
        out.step_pats.push(ev.pat_id);
        out.grid_win_bps += ev.win_bps;
        if !ev.any_pay {
            break;
        }
        collapse_and_refill(&mut grid, &ev.pay_sym, &mut cur);
    }

    if stars >= SN_TRIGGER {
        let mut prizes = PRIZES;
        // Fisher-Yates shuffle with draw_range (mirrors Solidity loop)
        for i in (1..12u8).rev() {
            let j = cur.draw_range(i + 1);
            prizes.swap(i as usize, j as usize);
        }
        out.prizes = Some(prizes);
    }
    out
}

// ---------------- picks / gamble / payout ----------------

/// Sum of picked supernova prizes, +10% if temple artifact.
pub fn picks_bps(prizes: &[u16; 12], picks: &[u8; SN_PICKS], artifacts: u8) -> u64 {
    let mut sum: u64 = picks.iter().map(|&p| prizes[p as usize] as u64).sum();
    if artifacts & ART_TEMPLE != 0 {
        sum = sum * 11 / 10;
    }
    sum
}

/// Validate picks: 5 distinct indices in [0,12).
pub fn valid_picks(picks: &[u8; SN_PICKS]) -> bool {
    let mut seen = [false; 12];
    for &p in picks.iter() {
        if p as usize >= 12 || seen[p as usize] {
            return false;
        }
        seen[p as usize] = true;
    }
    true
}

/// THE single payout function: every settlement path routes through here.
pub fn payout_for(wager: u64, win_bps: u64) -> u64 {
    let p = (wager as u128 * win_bps as u128) / 10_000;
    let cap = wager as u128 * MAX_PAYOUT_X as u128;
    std::cmp::min(p, cap) as u64
}

/// Fair coin from a seed byte (256 divisible by 2, no rejection needed).
pub fn coin_flip(seed: [u8; 32]) -> bool {
    seed[0] % 2 == 0
}

// ---------------- tests ----------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paytable_tiers() {
        assert_eq!(pay_for(0, 8), 1650);
        assert_eq!(pay_for(0, 10), 4125);
        assert_eq!(pay_for(0, 12), 16500);
        assert_eq!(pay_for(6, 12), 412500);
        assert_eq!(pay_for(3, 7), 0);
    }

    #[test]
    fn pattern_cruz_matches() {
        // cruz cells (col-major): 11,12,13,7,17 -> all copper
        let mut grid = [STAR; CELLS];
        for &c in &[11u8, 12, 13, 7, 17] {
            grid[c as usize] = 0;
        }
        assert!(match_pattern(&grid, &PATTERNS[0].1));
        assert!(!match_pattern(&grid, &PATTERNS[1].1));
    }

    #[test]
    fn pattern_rejects_stars() {
        let grid = [STAR; CELLS];
        for &(_, cells) in PATTERNS.iter() {
            assert!(!match_pattern(&grid, &cells));
        }
    }

    #[test]
    fn payout_cap() {
        assert_eq!(payout_for(1_000, 10_000), 1_000); // 1x
        assert_eq!(payout_for(1_000, 100_000_000), 1_000_000); // capped at 1000x
    }

    #[test]
    fn picks_validation() {
        assert!(valid_picks(&[0, 1, 2, 3, 4]));
        assert!(!valid_picks(&[0, 1, 2, 3, 3])); // duplicate
        assert!(!valid_picks(&[0, 1, 2, 3, 12])); // out of range
    }

    #[test]
    fn spin_is_deterministic() {
        let seed = [7u8; 32];
        let a = run_spin(seed, 0);
        let b = run_spin(seed, 0);
        assert_eq!(a.grid_win_bps, b.grid_win_bps);
        assert_eq!(a.grids, b.grids);
        assert_eq!(a.prizes, b.prizes);
    }

    #[test]
    fn draw_symbol_stays_in_range() {
        let mut cur = Cursor::new([42u8; 32]);
        for _ in 0..10_000 {
            assert!(cur.draw_symbol() <= STAR);
        }
    }
}
