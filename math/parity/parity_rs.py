#!/usr/bin/env python3
"""Parity: starforge-math (Rust, keccak cursor) vs simulator/parity.py (Python).
15 seeds x {0x00, 0x07} artifacts. Compares grid_win_bps, step_wins_bps,
step_pats, step_count, stars, prizes, grid0. Writes PARITY.md.
"""
import json
import subprocess
import sys

from eth_hash.auto import keccak

SEEDS = [keccak(f"starforge-parity-{i}".encode()).hex() for i in range(1, 16)]
ARTIFACT_SETS = [0x00, 0x07]

SIM = "/home/hatch/workspace/chain-jam/starforge/simulator/parity.py"
HARNESS = "/home/hatch/workspace/colosseum/starforge-arena/math/parity-harness/target/release/parity-harness"
PY = "/tmp/sfevm/bin/python"


def parse_py(out):
    d = {}
    for line in out.strip().splitlines():
        k, v = line.split("=", 1)
        d[k] = v
    return {
        "grid_win_bps": int(d["grid_win_bps"]),
        "step_count": int(d["step_count"]),
        "step_wins_bps": json.loads(d["step_wins_bps"].replace("'", '"')),
        "step_pats": json.loads(d["step_pats"].replace("'", '"')),
        "stars": int(d["stars"]),
        "prizes": json.loads(d["prizes"].replace("'", '"')),
        "grid0": json.loads(d["grid0"].replace("'", '"')),
    }


def main():
    rows = []
    fails = 0
    total = 0
    for i, seed in enumerate(SEEDS, 1):
        for arts in ARTIFACT_SETS:
            py_out = subprocess.run(
                [PY, SIM, seed, str(arts)], capture_output=True, text=True, check=True
            ).stdout
            rs_out = subprocess.run(
                [HARNESS, seed, str(arts)], capture_output=True, text=True, check=True
            ).stdout
            py, rs = parse_py(py_out), json.loads(rs_out)
            ok = True
            for k in ("grid_win_bps", "step_count", "stars"):
                total += 1
                if py[k] != rs[k]:
                    ok = False
                    fails += 1
            for k in ("step_wins_bps", "step_pats", "prizes", "grid0"):
                total += 1
                if list(py[k]) != list(rs[k]):
                    ok = False
                    fails += 1
            rows.append((i, seed[:8], f"{arts:#04x}", "PASS" if ok else "FAIL",
                         py["grid_win_bps"], rs["grid_win_bps"], py["stars"]))
            print(f"seed{i:02d} arts={arts:#04x} {'PASS' if ok else 'FAIL'} "
                  f"grid_win={py['grid_win_bps']} stars={py['stars']}")

    md = ["# Paridad math ↔ Python (crate `starforge-math`)",
          "",
          "Harness: `math/parity-harness` con cursor keccak (feature `parity-keccak`),",
          "byte-idéntico al `Cursor` de `simulator/parity.py`. El programa on-chain usa",
          "siempre sha256 (nativo Solana); la paridad valida el *algoritmo*, no el hash.",
          "",
          f"Resultado: **{total - fails}/{total} checks**, {len(rows) - sum(1 for r in rows if r[3]=='FAIL')}/{len(rows)} semillas PASS.",
          "",
          "| # | seed | artifacts | resultado | grid_win_bps (py) | grid_win_bps (rs) | stars |",
          "|---|------|-----------|-----------|-----------------|-----------------|-------|"]
    for i, s8, arts, res, gpy, grs, stars in rows:
        md.append(f"| {i} | `{s8}…` | {arts} | {res} | {gpy} | {grs} | {stars} |")
    md += ["", f"_Generado: 15 semillas × 2 artifact sets. Fallos: {fails}._", ""]
    with open("/home/hatch/workspace/colosseum/starforge-arena/math/parity/PARITY.md", "w") as f:
        f.write("\n".join(md))
    print(f"\n{total - fails}/{total} checks passed")
    sys.exit(1 if fails else 0)


if __name__ == "__main__":
    main()
