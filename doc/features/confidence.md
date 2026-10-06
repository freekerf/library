# Níveis de confiança

Toda receita declara `confidence.level`. O app mostra o nível ao usuário e
**nunca** apresenta uma receita `estimated` como garantida.

| `level` | Rótulo (pt) | Significa | Campos |
| --- | --- | --- | --- |
| `tested` | testado | Conferida com grade de teste | `author` (obrig.), `evidence` (obrig., ≥ 1: caminho em `evidence/` ou URL https), `date` (`YYYY-MM-DD`) |
| `community` | comunidade | Relatada, sem evidência | `reported_by` |
| `estimated` | estimado | Derivada (ex.: escala de potência) ou valor de partida | `method` (obrig.), `derived_from` (`<material-id>/<recipe-id>`) |

```toml
confidence = { level = "tested", author = "Ana", evidence = ["evidence/mdf-3mm/diode-10w-cut-ana.jpg"], date = "2026-10-01" }
confidence = { level = "community", reported_by = "fórum X" }
confidence = { level = "estimated", method = "Escala de potência 10 W → 5 W", derived_from = "basswood-3mm/diode-10w-cut" }
```

Validação (`confidence-evidence`): evidência local precisa existir no
repositório (caminho relativo, sem `..`), URLs só `https://`, e `derived_from`
precisa apontar para uma receita existente.

Estado atual dos dados: os 10 materiais curados são `estimated` (valores de
partida, sem grade de teste); os importados do LaserGRBL são `community`.
