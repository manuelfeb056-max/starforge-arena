# App — plan de adaptación del frontend

Se reutiliza el frontend Vite+TS de `~/workspace/chain-jam/starforge/frontend/`
**sin tocar el juego visual**: `game.ts`, `render.ts`, `audio.ts`, `crucible.ts`,
`i18n.ts`, `style.css` se copian tal cual.

## Lo que cambia (capa chain únicamente)

| Archivo original | Cambio |
|---|---|
| `src/host.ts` (bridge penpal al host del jam) | **Se elimina.** No hay host en Colosseum. |
| `src/abi.ts` (ABI del contrato EVM) | **Se reemplaza** por cliente Anchor generado (`@coral-xyz/anchor` + IDL). |
| `src/engine.ts` (math pura TS para demo) | Se mantiene para animación/instant-reveal; la verdad final es el programa. |
| `src/sdk/` | **Se reemplaza** por `@solana/web3.js` + `@solana/wallet-adapter` (Phantom). |

## Nuevo `src/solana.ts` (a construir en semana 2)
- Conexión a devnet, wallet-adapter.
- `startSession(wager, artifacts)` → firma y envía `start_session`.
- Poll de la cuenta `Session` hasta `stage == PICKS/DONE` (o suscripción por websockets).
- `submitPicks(picks, gamble)` → firma el jugador.
- Lee eventos `SpinRevealed` / `SessionSettled` para animar el reveal exacto
  (grids, step_pats, prizes) — la UI ya sabe animar esto desde `engine.ts`.
- Crank permissionless: cualquiera puede llamar `reveal_spin`/`settle_gamble`;
  la app lo hace automáticamente tras `request_slot` (bono UX: "sin esperas").

## Dependencias nuevas (semana 2)
```
@coral-xyz/anchor @solana/web3.js
@solana/wallet-adapter-base @solana/wallet-adapter-react
@solana/wallet-adapter-wallets @solana/wallet-adapter-react-ui
```

## Demo UX (lo que ve el jurado)
1. Conecta Phantom (devnet) → 2. Elige apuesta y artefactos → 3. SPIN →
   la app hace crank automático → animación del grid con el reveal on-chain →
   supernova → picks → gamble → payout. Todo con links al explorer de devnet.
