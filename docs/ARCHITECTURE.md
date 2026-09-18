# Arquitectura — Starforge Arena (Solana)

## Componentes

```
┌─────────────┐      ┌──────────────────┐      ┌─────────────────┐
│   app/      │      │ programs/        │      │ math/           │
│  (Vite+TS)  │─────▶│ starforge        │─────▶│ starforge-math  │
│ wallet-     │ tx   │ (Anchor)         │ call │ (crate puro)    │
│ adapter     │      │ House · Session  │      │ 6×5, RNG, pays  │
└─────────────┘      └──────────────────┘      └─────────────────┘
        │                       │                        │
        │              ┌────────┴────────┐        ┌────────┴────────┐
        └─────────────▶│ SlotHashes     │        │ scripts/        │
                       │ (RNG devnet)   │        │ verify_rtp.py   │
                       └────────────────┘        └─────────────────┘
```

## Programa Anchor `starforge`

**Cuentas:**
- `House` (PDA): `authority`, `fee_bps` (0 en MVP), bump. Vault = la propia PDA (lamports).
- `Session` (PDA, seeds `[b"session", player, session_id]`): jugador, apuesta, bitmask
  de artefactos, etapa, `request_slot`, datos del reveal (grids, step_wins, step_pats,
  prizes, picks), payout final. El vault de la sesión retiene `wager` hasta liquidar.

**Instrucciones (flujo = el del contrato EVM, adaptado a slots):**
1. `initialize_house` — crea House PDA.
2. `start_session(wager, artifacts, session_id)` — el jugador transfiere `wager`
   lamports a la Session PDA; se fija `request_slot = slot_actual + 10`.
   Compromiso de randomness: el seed será el hash de ese slot futuro.
3. `reveal_spin` (permissionless, tras `request_slot`) — lee `SlotHashes` sysvar,
   construye el seed, ejecuta la matemática (grid 30 celdas, cascadas cap 2,
   patrones, conteo de estrellas). Si `stars >= 4`: mezcla los 12 premios de la
   supernova → etapa PICKS. Si no: liquida directo.
4. `submit_picks([5 índices distintos], gamble: bool)` — solo el jugador; suma
   `picks_bps` (+10% si temple). Si `gamble == false`: liquida. Si `true`:
   etapa GAMBLE, `request_slot = slot_actual + 10`.
5. `settle_gamble` (permissionless, tras slot) — moneda justa del seed → liquida.
6. `settle` interno — `payout = min(wager * total_bps / 10000, wager * 1000x)`;
   transfiere al jugador, emite evento `SessionSettled` con el reveal completo,
   cierra la Session PDA (rent al jugador).

**Eventos (para el frontend y el verificador):**
- `SpinRevealed { session, player, seed, grids, step_pats, supernova: bool }`
- `SessionSettled { session, player, wager, total_bps, payout }`

## Crate `starforge-math`

Port 1:1 de `math-spec.json` / `StarforgeGame.sol`, sin dependencias de Solana:
rejection sampling (`SYMBOL_REJECT = 65514`), paytables en bps, 4 constelaciones,
supernova (12 premios, shuffle Fisher-Yates), cascadas con gravedad por columna,
cap 1000x, `RTP_BPS = 9760`. Se usa dentro del programa Y en tests de paridad.

## Cómo se demuestra el RTP publicado

1. **Constante on-chain:** `RTP_BPS = 9760` declarada en el programa.
2. **Paridad:** `math/parity/` — el crate Rust debe reproducir exactamente las
   15 semillas de `simulator/parity.py` del proyecto original (mismo algoritmo).
3. **Monte Carlo heredado:** 10M sims del proyecto original: 93.94% base.
   (La matemática no cambió; el port solo debe pasar paridad.)
4. **Verificador independiente:** `scripts/verify_rtp.py` — lee sesiones
   liquidadas de devnet (eventos `SessionSettled`), re-ejecuta cada sesión desde
   su `seed` con el simulador Python y compara `total_bps` on-chain vs
   re-ejecutado. Cualquiera puede correrlo: confianza = 0 requerida.

## RNG: honesto sobre el MVP

- **Devnet/MVP:** `SlotHashes` — el seed es el hash de un slot futuro fijado al
  abrir la sesión. Verificable, sin oráculo, sin costo. Suficiente para demo y jurado.
- **Mainnet (fuera del MVP):** migrar a Orao VRF o Switchboard Randomness;
  la interfaz del programa ya aísla el RNG en un módulo para ese cambio.
