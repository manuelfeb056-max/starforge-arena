# Starforge Arena — Colosseum Crypto World's Fair

**Arcade on-chain verificablemente justo, con RTP publicado — nativo en Solana.**

Scatter-pay 6×5 "cosmic-forge slot": forja minerales, dispara la supernova, doble o nada.
Toda la matemática es determinista y abierta: cualquiera puede re-ejecutar cualquier
sesión desde su semilla on-chain y verificar el RTP declarado.

- **Track objetivo:** Solana ($100K, 10 premios de $10K) — ver `docs/ROUTE_DECISION.md`
- **Deadline:** 12 oct 2026, 23:59 PDT
- **Cuenta:** `nueve` (registrada 2026-09-18)
- **RTP declarado:** 97.60% (base, sin artefactos) / 97.59% (con artefactos) — Monte Carlo 10M
- **Repo público:** https://github.com/manuelfeb056-max/starforge-arena

## Estructura

```
programs/starforge/   Programa Anchor: House + Session, flujo start→spin→picks→gamble→settle
math/starforge-math/  Crate Rust puro: port 1:1 de math-spec.json (sin dependencias de Solana)
math/parity/          Plan de paridad programa↔simulador Python (15/15)
app/                  Plan de adaptación del frontend Vite existente → web3.js + Anchor
scripts/              verify_rtp.py — verificador independiente de RTP desde sesiones on-chain
docs/                 Decisiones, arquitectura, hitos
```

## Origen

Port de [STARFORGE (Chain Jam)](../../chain-jam/starforge/) — contrato Solidity
`StarforgeGame.sol` verificado: paridad contrato/simulador 15/15, Monte Carlo 10M
sin artefactos. La matemática NO cambia; solo cambia la capa de ejecución (EVM → SVM).

## Reglas del build

- $0 gasto. Sin deploys a mainnet (devnet/test-validator únicamente).
- Sin wallets reales, sin transacciones con fondos reales.
- Randomness en devnet: `SlotHashes` sysvar (verificable on-chain). Mainnet: upgrade a VRF (Orao/Switchboard) — documentado, fuera del MVP.
