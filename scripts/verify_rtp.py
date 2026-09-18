#!/usr/bin/env python3
"""
verify_rtp.py — verificador INDEPENDIENTE del RTP de Starforge Arena.

Lee sesiones liquidadas de devnet (eventos SessionSettled), re-ejecuta cada
sesión desde su `seed` on-chain con el simulador Python de referencia y compara
el `total_bps` on-chain contra el re-ejecutado.

Uso:
    python3 scripts/verify_rtp.py --rpc https://api.devnet.solana.com \
        --program <PROGRAM_ID> [--limit 500]

Cualquiera puede correrlo: confianza requerida = 0.
"""
import argparse
import json
import sys
from pathlib import Path

# Reutiliza el simulador auditado del proyecto original (no se duplica lógica).
SIM_DIR = Path.home() / "workspace/chain-jam/starforge/simulator"
sys.path.insert(0, str(SIM_DIR))

DECLARED_RTP_BPS = 9760  # debe coincidir con starforge-math::RTP_BPS
TOLERANCE_BPS = 150      # tolerancia estadística para N sesiones (se ajusta por N)


def fetch_settled_sessions(rpc: str, program_id: str, limit: int):
    """TODO (semana 3): paginar getSignaturesForAddress del program id,
    parsear logs de eventos SessionSettled -> {session, player, wager,
    total_bps, payout, seed}. Requiere `solana-py` o RPC crudo."""
    raise NotImplementedError("conectar a devnet en semana 3")


def replay_session(seed_hex: str, artifacts: int):
    """TODO (semana 2): importar sim.py y re-ejecutar run_spin con la semilla.
    NOTA: sim.py usa su propio RNG; para paridad exacta el harness debe
    alimentarse con el seed on-chain. Ver math/parity/README.md."""
    raise NotImplementedError("harness de paridad en semana 2")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--rpc", default="https://api.devnet.solana.com")
    ap.add_argument("--program", required=True)
    ap.add_argument("--limit", type=int, default=500)
    args = ap.parse_args()

    sessions = fetch_settled_sessions(args.rpc, args.program, args.limit)
    mismatches = 0
    total_wagered = 0
    total_paid = 0
    for s in sessions:
        total_wagered += s["wager"]
        total_paid += s["payout"]
        expected = replay_session(s["seed"], s["artifacts"])
        if expected != s["total_bps"]:
            mismatches += 1
            print(f"MISMATCH {s['session']}: on-chain={s['total_bps']} replay={expected}")

    realized_bps = int(10_000 * total_paid / total_wagered) if total_wagered else 0
    print(f"sesiones verificadas: {len(sessions)}")
    print(f"mismatches: {mismatches}")
    print(f"RTP realizado: {realized_bps/100:.2f}%  |  declarado: {DECLARED_RTP_BPS/100:.2f}%")
    ok = mismatches == 0 and abs(realized_bps - DECLARED_RTP_BPS) <= TOLERANCE_BPS
    print("VEREDICTO:", "OK — verificablemente justo" if ok else "FALLA — investigar")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
