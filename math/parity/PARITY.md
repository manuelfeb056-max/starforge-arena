# Paridad math ↔ Python (crate `starforge-math`)

Harness: `math/parity-harness` con cursor keccak (feature `parity-keccak`),
byte-idéntico al `Cursor` de `simulator/parity.py`. El programa on-chain usa
siempre sha256 (nativo Solana); la paridad valida el *algoritmo*, no el hash.

Resultado: **210/210 checks**, 30/30 semillas PASS.

| # | seed | artifacts | resultado | grid_win_bps (py) | grid_win_bps (rs) | stars |
|---|------|-----------|-----------|-----------------|-----------------|-------|
| 1 | `b3568278…` | 0x00 | PASS | 10725 | 10725 | 0 |
| 1 | `b3568278…` | 0x07 | PASS | 10725 | 10725 | 0 |
| 2 | `34640682…` | 0x00 | PASS | 10725 | 10725 | 0 |
| 2 | `34640682…` | 0x07 | PASS | 10725 | 10725 | 0 |
| 3 | `fe8a734e…` | 0x00 | PASS | 4125 | 4125 | 0 |
| 3 | `fe8a734e…` | 0x07 | PASS | 4125 | 4125 | 0 |
| 4 | `bcaff73f…` | 0x00 | PASS | 5775 | 5775 | 2 |
| 4 | `bcaff73f…` | 0x07 | PASS | 5775 | 5775 | 2 |
| 5 | `a248f2f2…` | 0x00 | PASS | 8250 | 8250 | 0 |
| 5 | `a248f2f2…` | 0x07 | PASS | 8250 | 8250 | 0 |
| 6 | `9950d3a6…` | 0x00 | PASS | 0 | 0 | 0 |
| 6 | `9950d3a6…` | 0x07 | PASS | 0 | 0 | 0 |
| 7 | `b182dc28…` | 0x00 | PASS | 9900 | 9900 | 2 |
| 7 | `b182dc28…` | 0x07 | PASS | 9900 | 9900 | 2 |
| 8 | `1963f65f…` | 0x00 | PASS | 0 | 0 | 1 |
| 8 | `1963f65f…` | 0x07 | PASS | 0 | 0 | 1 |
| 9 | `19b9053b…` | 0x00 | PASS | 4125 | 4125 | 3 |
| 9 | `19b9053b…` | 0x07 | PASS | 4125 | 4125 | 3 |
| 10 | `ed020a02…` | 0x00 | PASS | 13200 | 13200 | 1 |
| 10 | `ed020a02…` | 0x07 | PASS | 13200 | 13200 | 1 |
| 11 | `78d5622c…` | 0x00 | PASS | 2475 | 2475 | 1 |
| 11 | `78d5622c…` | 0x07 | PASS | 2475 | 2475 | 1 |
| 12 | `2e61c760…` | 0x00 | PASS | 0 | 0 | 4 |
| 12 | `2e61c760…` | 0x07 | PASS | 0 | 0 | 4 |
| 13 | `3b9c617c…` | 0x00 | PASS | 1650 | 1650 | 1 |
| 13 | `3b9c617c…` | 0x07 | PASS | 1650 | 1650 | 1 |
| 14 | `54253a01…` | 0x00 | PASS | 0 | 0 | 2 |
| 14 | `54253a01…` | 0x07 | PASS | 0 | 0 | 2 |
| 15 | `739e8422…` | 0x00 | PASS | 10725 | 10725 | 1 |
| 15 | `739e8422…` | 0x07 | PASS | 10725 | 10725 | 1 |

_Generado: 15 semillas × 2 artifact sets. Fallos: 0._
