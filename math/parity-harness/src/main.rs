//! Parity harness: `run_spin_keccak(seed, artifacts)` -> JSON on stdout.
//! Usage: parity-harness <seed_hex_64> <artifacts_u8>
//! Mirrors `simulator/parity.py` output fields. Keccak cursor matches the
//! Python/EVM reference byte-for-byte (algorithm parity, not the hash itself).

use starforge_math::run_spin_keccak;
use std::env;

fn hex_to_seed(s: &str) -> [u8; 32] {
    assert_eq!(s.len(), 64, "seed must be 64 hex chars");
    let mut seed = [0u8; 32];
    for i in 0..32 {
        seed[i] = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("bad hex");
    }
    seed
}

fn main() {
    let args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 3, "usage: parity-harness <seed_hex> <artifacts>");
    let seed = hex_to_seed(&args[1]);
    let artifacts: u8 = args[2].parse().expect("artifacts must be u8");

    let o = run_spin_keccak(seed, artifacts);
    let v = serde_json::json!({
        "grid_win_bps": o.grid_win_bps,
        "step_wins_bps": o.step_wins_bps,
        "step_pats": o.step_pats,
        "step_count": o.grids.len(),
        "stars": o.stars,
        "prizes": o.prizes.map(|p| p.to_vec()).unwrap_or(vec![0u16; 12]),
        "grid0": o.grids.first().map(|g| g.to_vec()).unwrap_or_default(),
    });
    println!("{}", v);
}
