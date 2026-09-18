# Paridad math ↔ programa

La matemática NO se rediseña: es transliteración verificada.

## Fuentes de verdad (ya existen, no se tocan)
- `~/workspace/chain-jam/starforge/math-spec.json` — spec canónica
- `~/workspace/chain-jam/starforge/simulator/sim.py` — simulador Python
- `~/workspace/chain-jam/starforge/simulator/parity.py` — 15 semillas de paridad
  contrato↔simulador (todas pasan en el proyecto original)

## Plan de paridad para el crate Rust
1. `cargo test -p starforge-math` — unit tests del crate (paytables, patrones,
   picks, cap, determinismo). Ya incluidos en `src/lib.rs`.
2. **Paridad algorítmica (15/15):** extender `parity.py` (o un `parity_rs.py`
   nuevo) que, para cada semilla, ejecute `sim.py` y compare contra un binario
   Rust mínimo (`math/parity-harness/`) que imprima `run_spin(seed)` en JSON.
   Criterio: `grid_win_bps`, `grids`, `step_pats`, `prizes` idénticos en las
   15 semillas.
   - Nota honesta: el contrato EVM usa keccak256 y el crate usa sha256 para el
     cursor. La paridad byte-a-byte contra `sim.py` requiere que el harness de
     paridad use el mismo hash que el simulador Python para esas 15 semillas
     (inyectar el hash como parámetro del harness). El algoritmo de muestreo
     (límites de rechazo, orden de draws) es idéntico, que es lo que garantiza
     el mismo RTP estadístico.
3. **Monte Carlo heredado:** 10M sims ya hechos en el proyecto original
   (93.94% base). No se repiten salvo que la paridad falle; si se repiten, usar
   `frontend/scripts/montecarlo.mjs` o `sim.py`.

## Hito de aceptación
`math/parity/PARITY.md` con tabla semilla→`total_bps` (Python vs Rust): 15/15
iguales antes de tocar devnet.
