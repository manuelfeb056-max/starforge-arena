# Hitos — Starforge Arena → Colosseum (deadline 12 oct 2026)

Cuenta `nueve` registrada. Submissions: repo público + demo + writeup.

## Semana 1 — 18→24 sep — Matemática y compilación
- [x] Scaffold del repo (este commit)
- [ ] Instalar toolchain: `rustup` + `cargo`, `anchor-cli` (0.31.1)
- [ ] `cargo test -p starforge-math` verde
- [ ] `anchor build` del programa verde (sin warnings bloqueantes)
- [ ] Paridad 15/15: `math/parity/PARITY.md` (Python vs Rust)

## Semana 2 — 25 sep→1 oct — Flujo on-chain local
- [ ] Tests Anchor (TS) del ciclo completo en `solana-test-validator`:
      start → reveal_spin → picks → gamble → settle, con asserts de payout
- [ ] `app/src/solana.ts`: cliente Anchor + wallet-adapter
- [ ] Frontend corriendo contra validator local, crank automático
- [ ] `gh auth login` (Mannuel) → repo público `nueve/starforge-arena`

## Semana 3 — 2→8 oct — Devnet y verificación
- [ ] Deploy a devnet (program id real, actualizar `declare_id!`)
- [ ] Demo end-to-end en devnet con Phantom (video para el jurado)
- [ ] `scripts/verify_rtp.py` funcional contra devnet
- [ ] N sesiones de prueba en devnet → RTP realizado dentro de tolerancia

## Semana 4 — 9→12 oct — Submission
- [ ] README público pulido: RTP, arquitectura, links a devnet explorer
- [ ] Video demo (≤3 min) + writeup para Colosseum Arena
- [ ] Submit en `colosseum.com/arena/hackathon` antes del 12 oct 23:59 PDT
- [ ] Buffer: 12 oct solo para imprevistos

## Bloqueadores externos (no-code)
1. `gh auth login` en la VM — lo hace Mannuel (repo público exigido por Colosseum)
2. Instalación de toolchain Rust/Anchor — se hace en semana 1 (gratis, ~10 min)

## Criterios del jurado → dónde atacamos
- **Funcionalidad:** ciclo completo jugable en devnet + tests
- **Impacto:** RTP publicado + verificador independiente (confianza cero)
- **Novedad:** arcade verificablemente justo, reveal on-chain animado
- **UX:** frontend existente pulido + crank automático (sin esperas)
- **Open source:** todo MIT en el repo público
- **Plan de negocio:** house con `fee_bps` configurable + cap 1000x (riesgo acotado)
