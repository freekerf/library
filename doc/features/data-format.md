# Formato dos dados (schema v1)

Fonte da verdade: `crates/freekerf-library/src/model/`. Schemas gerados:
[`schema/v1/material.schema.json`](../../schema/v1/material.schema.json),
[`machine.schema.json`](../../schema/v1/machine.schema.json),
[`hazards.schema.json`](../../schema/v1/hazards.schema.json),
[`index.schema.json`](../../schema/v1/index.schema.json).

Identificadores: `kebab-case` ASCII (`^[a-z0-9]+(-[a-z0-9]+)*$`); o arquivo se
chama `<id>.toml`. Chaves e valores em inglês. Campos desconhecidos são erro.

## Material — `data/materials/<categoria>/<id>.toml`

| Campo | Tipo | Obrig. | Unidade / notas |
| --- | --- | --- | --- |
| `id` | string | sim | = nome do arquivo |
| `name` | string | sim | 1–120 caracteres |
| `category` | enum | sim | `wood`, `engineered_wood`, `acrylic`, `leather`, `paper`, `textile`, `anodized_metal`, `coated_metal`, `metal`, `stone`, `glass`, `ceramic`, `plastic`, `foam`, `rubber`, `cork`, `other` |
| `thickness_mm` | número | não | mm (0,01–100); omita em materiais só de superfície |
| `supplier` | string | não | fornecedor/produto |
| `notes` | string | não | |
| `recipes` | lista | sim (≥ 1) | ver abaixo |

## Receita — `[[recipes]]`

| Campo | Tipo | Obrig. | Unidade / notas |
| --- | --- | --- | --- |
| `id` | string | sim | único no material |
| `operation` | enum | sim | `cut`, `engrave_vector`, `engrave_raster` |
| `laser` | `{ kind, power_w }` | sim | `kind`: `diode`, `co2`, `fiber`; `power_w`: potência **óptica real** em W |
| `machine` | string | não | id de um perfil em `data/machines` |
| `speed_mm_min` | inteiro | sim | mm/min |
| `power_min_pct` / `power_max_pct` | número | sim | % (0–100), mín ≤ máx |
| `passes` | inteiro | sim | 1–100 |
| `air_assist` | bool | não | omitido = indiferente |
| `focus_offset_mm` | número | não | mm a partir da superfície; negativo = para dentro |
| `line_interval_mm` | número | não | mm, só `engrave_raster` |
| `dpi` | inteiro | não | só `engrave_raster`; se junto com `line_interval_mm`, precisam concordar (±10 %) |
| `dithering` | enum | não | só `engrave_raster`: `none`, `threshold`, `grayscale`, `floyd_steinberg`, `jarvis`, `stucki`, `atkinson`, `ordered` |
| `confidence` | tabela | sim | ver [confidence.md](confidence.md) |
| `source` | `{ name, url?, license, reference? }` | não | origem de dados de terceiros; `license` em SPDX |
| `notes` | string | não | |

Exemplo:

```toml
id = "basswood-3mm"
name = "Basswood sheet"
category = "wood"
thickness_mm = 3

[[recipes]]
id = "diode-10w-cut"
operation = "cut"
laser = { kind = "diode", power_w = 10 }
machine = "generic-diode-10w-grbl"
speed_mm_min = 400
power_min_pct = 100
power_max_pct = 100
passes = 1
air_assist = true
focus_offset_mm = -1
confidence = { level = "estimated", method = "…" }
```

## Perfil de máquina — `data/machines/<fabricante>/<id>.toml`

| Campo | Tipo | Obrig. | Unidade / notas |
| --- | --- | --- | --- |
| `id`, `manufacturer`, `model` | string | sim | `Generic` para perfis de referência |
| `work_area` | `{ x_mm, y_mm, z_mm? }` | sim | mm |
| `firmware` | `{ kind, min_version? }` | sim | `grbl`, `grbl_hal`, `smoothieware`, `marlin` |
| `origin` | enum | sim | `front_left`, `front_right`, `rear_left`, `rear_right`, `center` |
| `features` | `{ air_assist, rotary, camera, z_axis }` | sim | bools explícitos |
| `laser` | `{ kind, power_w }` | sim | potência **óptica real** |
| `electrical_power_w` | número | não | potência elétrica/de marketing, W |
| `max_speed_mm_min` | inteiro | não | mm/min |
| `settings` | lista `{ key, value, description }` | não | `$$` recomendados (só Grbl/grblHAL), ex. `$32 = 1` |
| `notes` | string | não | |

## Lista de perigos

Ver [safety.md](safety.md).
