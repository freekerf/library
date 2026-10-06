# Validação

`cargo run -- validate` (CI em todo PR). Falha com qualquer erro; avisos só
falham com `--deny-warnings`. `--format json` emite os diagnósticos em JSON.

## Camada 1 — arquivo (`Loader`, `DocumentCheck`)

| Verificação | Nível | O quê |
| --- | --- | --- |
| `toml` | erro | TOML inválido, chave duplicada |
| `toml-lint` | erro | CRLF, tabulação, falta/excesso de newline final, nome do arquivo ≠ `id` |
| `schema` | erro | JSON Schema v1: tipos, enums, campos obrigatórios/desconhecidos, padrões de id, faixas absolutas (ex.: potência 0–100 %) |
| `model` | erro | falha de desserialização não coberta pelo schema |
| `missing-file` | erro | `data/safety/hazards.toml` ausente |

## Camada 2 — semântica (`Validator`, `Rule`)

| Regra | Nível | O quê |
| --- | --- | --- |
| `unique-ids` | erro | ids de material, máquina, perigo e receita (por material) únicos |
| `unique-ids` | aviso | duas receitas iguais em operação + laser + máquina + registro de origem |
| `power-range` | erro | `power_min_pct ≤ power_max_pct`, `power_max_pct > 0` |
| `plausible-process` | erro | potência óptica plausível por tecnologia (diodo 0,5–60 W, CO₂ 10–300 W, fibra 10–500 W) e velocidade (diodo ≤ 60 000, CO₂ ≤ 120 000, fibra ≤ 1 200 000 mm/min) |
| `plausible-process` | aviso | mais de 20 passes; corte em material sem `thickness_mm` |
| `raster-params` | erro | `dpi`/`line_interval_mm`/`dithering` só em `engrave_raster`; `dpi` e intervalo coerentes |
| `machine-references` | erro / aviso | máquina inexistente ou de outra tecnologia (erro); potência difere > 10 % (aviso) |
| `machine-settings` | erro / aviso | `$$` só para Grbl/grblHAL, chaves únicas, potência plausível, elétrica ≥ óptica (erros); falta `$32 = 1` (aviso) |
| `confidence-evidence` | erro | ver [confidence.md](confidence.md) |
| `source-license` | erro | `source.license` fora da allow-list compatível com GPL-3.0 |
| `hazards` | erro | material casa com perigo `block`; palavras-chave não normalizadas |

Os avisos atuais vêm dos dados importados do LaserGRBL (cortes sem espessura
informada na origem) e servem de lista de curadoria.
