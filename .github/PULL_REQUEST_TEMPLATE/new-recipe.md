## Nova receita / material

<!-- Todos os campos são obrigatórios. PRs sem a foto da grade de teste entram
     como `community` (sem evidência), nunca como `tested`. -->

**Material** (nome, fornecedor, espessura em mm):

**Máquina e laser** (fabricante/modelo, tipo `diode`/`co2`/`fiber`, potência óptica real em W):

**Operação** (`cut` / `engrave_vector` / `engrave_raster`):

**Parâmetros** (com unidades):
- velocidade (mm/min):
- potência mín./máx. (%):
- passes:
- ar (sim/não):
- foco (mm, negativo = dentro do material):
- intervalo de linhas (mm) ou DPI, dithering (só raster):

**Nível de confiança**: `tested` / `community` / `estimated`

**Evidência** (obrigatória para `tested`): foto da grade de teste em `evidence/<material-id>/` ou link https, autor e data.

**Origem dos dados**: autoria própria, ou fonte + licença SPDX compatível com GPL-3.0 (não copie bibliotecas proprietárias, ex.: LightBurn ou fabricantes, sem licença explícita).

## Checklist

- [ ] Arquivo em `data/materials/<categoria>/<id>.toml`, nome do arquivo = `id`
- [ ] Unidades explícitas (os nomes dos campos terminam em `_mm_min`, `_pct`, `_mm`, `_w`)
- [ ] O material não está na lista de perigosos (`data/safety/hazards.toml`)
- [ ] `cargo run -- validate` sem erros
