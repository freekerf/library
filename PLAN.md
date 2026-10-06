# FreeKerf Library: plano

Biblioteca aberta de **materiais** (parâmetros de corte e gravação) e **perfis de máquina** do FreeKerf. É consumida pelo crate `freekerf-materials` do monorepo [`freekerf/freekerf`](https://github.com/freekerf/freekerf) e pode ser usada por qualquer software.

> **Status: marcos 1 e 2 implementados; marco 3 pronto para a primeira release** (falta só a tag e a integração no monorepo). Licença: GPL-3.0-or-later (ver [NOTICE](NOTICE)). Manutenção: [AGENTS.md](AGENTS.md).

## 1. O que precisa ser construído

### 1.1 Esquema de dados (`schema/`)
- JSON Schema versionado (`schema/v1/material.schema.json`, `machine.schema.json`). Os arquivos de dados ficam em TOML (legível em diff), validados contra o schema.
- **Material:** nome, categoria (madeira, acrílico, couro, papel, metal anodizado…), espessura, fornecedor opcional e **receitas** por tipo de laser (diodo 5/10/20/40 W, CO₂ 40–150 W, fibra), cada uma com operação (corte, gravação vetorial, gravação raster), velocidade, potência mín./máx., passes, ar, foco, intervalo de linhas/DPI, dithering recomendado.
- **Perfil de máquina:** fabricante/modelo, área útil, firmware (Grbl/grblHAL/Smoothie/Marlin), `$$` recomendados, origem, recursos (ar, rotary, câmera, Z), módulo laser e potência real.
- Unidades explícitas (mm/min, %, mm). Nada de valores "mágicos" sem unidade.

### 1.2 Nível de confiança das receitas
Cada receita declara seu nível: `testado` (com foto/grade de teste anexada e autor), `comunidade` (relatado, sem evidência) ou `estimado` (derivado por escala de potência). O app mostra o nível ao usuário e nunca apresenta uma receita `estimado` como garantida.

### 1.3 Importação
- Ferramenta (`tools/`, Python ou Rust) que importa a biblioteca de materiais do LaserGRBL (formato CSV/XML atual) e gera os arquivos no novo formato.
- Pesquisar fontes abertas compatíveis com GPL antes de importar qualquer dado de terceiros. Não copie bibliotecas proprietárias (ex.: as do LightBurn ou de fabricantes) sem licença explícita.

### 1.4 Contribuição e validação
- Template de PR para nova receita (com campos obrigatórios e foto da grade de teste).
- CI: validação de schema, unidades e faixas plausíveis (ex.: potência 0–100%), chaves únicas e lint de TOML.
- Release versionada (semver) que publica um pacote único (`library-vX.Y.Z.tar.zst` + índice JSON) consumido pelo app via download ou embutido no instalador.

### 1.5 Segurança
- Lista negra de materiais perigosos (PVC/vinil, policarbonato, fibra de vidro, couro com cromo…), com o motivo (gases tóxicos/corrosivos). O app bloqueia ou alerta ao selecionar um desses materiais.

## 2. Marcos
1. Schema v1 + 10 materiais de exemplo + CI de validação.
2. Importador da biblioteca do LaserGRBL.
3. Processo de contribuição + primeira release consumida pelo `freekerf-materials` (marco M2 do monorepo).

## 3. Perguntas (respondidas)
- ~~Licença própria para os dados?~~ Os dados poderão usar CC BY-SA 4.0 como padrão, com licença escolhida por quem submete; isso está no [roadmap](doc/roadmap.md) e **não** foi implementado. Hoje tudo segue GPL-3.0-or-later.
- ~~Telemetria opt-in?~~ Não haverá telemetria.

## 4. Onde está cada item
| Item | Implementação | Documentação |
| --- | --- | --- |
| 1.1 Schema | `crates/freekerf-library/src/model/`, `schema/v1/` | [data-format](doc/features/data-format.md) |
| 1.2 Confiança | `Confidence`, regra `confidence-evidence` | [confidence](doc/features/confidence.md) |
| 1.3 Importação | `import/lasergrbl/`, `data/materials/imported/lasergrbl/` | [import-lasergrbl](doc/features/import-lasergrbl.md) |
| 1.4 Contribuição e CI | `.github/`, `validate/`, `package/` | [validation](doc/features/validation.md), [packaging](doc/features/packaging.md), [contributing](doc/features/contributing.md) |
| 1.5 Segurança | `data/safety/hazards.toml`, `safety.rs` | [safety](doc/features/safety.md) |
