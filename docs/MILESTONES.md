# Hitos — Starforge Arena → Colosseum (deadline 12 oct 2026)

Cuenta `nueve` registrada. Submissions: repo público + demo + writeup.

## Semana 1 — 18→24 sep — Matemática y compilación
- [x] Scaffold del repo (este commit)
- [x] Toolchain instalado y consistente (18 sep): Agave/Solana CLI **2.1.0** + platform-tools **v1.43** (rustc 1.79.0) + `anchor-cli` **0.31.1** — todo en `~/workspace/.toolchain/`, $0
- [x] `cargo test -p starforge-math` verde — **7/7 pass**
- [x] `anchor build` verde (18 sep): programa compila a SBF — `target/deploy/starforge.so` (**234 KB**), IDL + tipos TS generados, sin warnings bloqueantes
- [x] Paridad 15/15: `math/parity/PARITY.md` (Python vs Rust) — **30/30 casos, 210/210 checks PASS**
- [x] Suite TS Anchor creada (`tests/starforge.ts`): ciclo completo start → reveal_spin → picks → gamble → settle con asserts de payout; **compila limpio** (`tsc --noEmit`), deps npm instaladas ($0)
- [x] Program ID local: `9zrUdECfgs7CC2tzgNeRXjEQA6MWvQofFv1Rzbc1oc9m` (keypair en `target/deploy/starforge-keypair.json`, solo test)
- [ ] Tests Anchor (TS) ejecutados en cluster — **BLOQUEADO por entorno**: `solana-test-validator` no se sostiene en este sandbox (muere a los ~1–2 min por timeout de gossip discovery, "Discover failed"; `agave-validator` directo se cuelga tras el syscheck). Programa, IDL y tests listos para correr en cuanto haya cluster estable: `anchor deploy` + `npx ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.ts`

## Semana 2 — 25 sep→1 oct — Flujo on-chain local
- [x] Tests Anchor (TS) del ciclo completo escritos en `tests/starforge.ts`:
      start → reveal_spin → picks → gamble → settle, con asserts de payout
- [ ] **Ejecutarlos** en `solana-test-validator` o devnet (pendiente de cluster estable)
- [ ] `app/src/solana.ts`: cliente Anchor + wallet-adapter
- [ ] Frontend corriendo contra validator local, crank automático
- [x] `gh auth login` (Mannuel) → hecho 18 sep ~10:16 AST — falta crear/pushear repo público `manuelfeb056-max/starforge-arena`

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
