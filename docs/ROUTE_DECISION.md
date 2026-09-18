# Decisión de ruta: Solana (Anchor) — NO Base (EVM)

## Veredicto: programa Anchor en Solana

### 1. El premio manda
Colosseum tiene un **track dedicado a Solana: $100K repartidos en 10 premios de $10K**.
Base no tiene track dedicado: compite solo por los premios globales
(Grand Prize $30K, 20×$15K) contra todos los ecosistemas. La probabilidad de
cobrar es objetivamente mayor en el track Solana.

### 2. Novedad (criterio de jurado)
El contrato Solidity ya existe y corre en Base **sin cambios**. Presentar eso es
"el mismo juego en otra EVM" — esfuerzo visible ≈ 0, novedad ≈ 0.
Un port real a Rust/Anchor demuestra ingeniería seria y encaja con el sesgo
"Solana-native" del jurado del track.

### 3. La matemática es portable
`StarforgeGame.sol` es 100% funciones puras (sin dependencias del host salvo RNG).
El port a un crate Rust puro es mecánico y verificable por paridad contra el
simulador Python ya existente. Riesgo técnico bajo, payoff alto.

### 4. Historia de "verificablemente justo" más fuerte en Solana
En Solana podemos publicar: programa open-source + semilla on-chain por sesión +
verificador independiente (`scripts/verify_rtp.py`) que re-ejecuta cualquier sesión
y comprueba el RTP declarado. Todo auditable sin confiar en nadie.

## Lo que se descarta (y por qué)
- **Base/EVM:** port trivial, sin track dedicado, historia débil.
- **Tempo:** stablecoins, pero el juego no necesita pagos en stablecoins; el track
  Tempo premia payment-infra, no gaming.

## Riesgos de la ruta Solana y mitigación
- **Randomness:** sin VRF nativo barato en devnet → MVP usa `SlotHashes` sysvar
  (hash de un slot futuro comprometido al abrir la sesión; verificable por cualquiera).
  Upgrade a Orao/Switchboard VRF documentado para mainnet, fuera del MVP.
- **Curva Anchor/Rust:** la lógica ya está escrita y testeada en Solidity/Python;
  el trabajo es transliteración + tests de paridad, no diseño nuevo.
