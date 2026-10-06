# Importação do LaserGRBL

O LaserGRBL (GPL-3.0, Diego Settimi) traz uma base de materiais
`StandardMaterials.psh`: um DataSet ADO.NET serializado em XML, tabela
`Materials` com `id, Visible, Model, Material, Thickness, Action (Cut/Engrave),
Power (%), Speed (mm/min), Cycles, Remarks`. A licença é compatível, então os
dados podem ser importados com atribuição.

Cópia fixada em `tools/lasergrbl/upstream/StandardMaterials.psh` (commit
`1f9337b3` do upstream) para a importação ser reproduzível offline.

## Executar

```sh
cargo run -- import lasergrbl --clean \
  --input tools/lasergrbl/upstream/StandardMaterials.psh
```

Gera `data/materials/imported/lasergrbl/<categoria>/lasergrbl-<material>[-<esp>mm].toml`
e `REPORT.md` (linhas ignoradas e motivo). O teste
`lasergrbl_import_is_reproducible` falha se os arquivos commitados divergirem
de uma importação nova.

## Conversão

| LaserGRBL | FreeKerf |
| --- | --- |
| `Material` + `Thickness` | um material por (nome normalizado, espessura); `,`→`.`, `mm` removido; `-`/vazio = sem espessura; texto não numérico vai para `notes` |
| `Model` | `laser.power_w` lido do nome (`Ortur LU7W (1.5W)` → 1,5 W; parênteses têm prioridade) ou de `tools/lasergrbl/models.toml`; tecnologia `diode` |
| `Action` | `Cut` → `cut` (mín = máx = Power); `Engrave` → `engrave_raster` (mín 0, máx = Power) |
| `Speed`, `Cycles` | `speed_mm_min`, `passes` |
| `Remarks` | `notes` |
| — | `confidence = community`, `source` com URL fixada, licença `GPL-3.0-or-later` e `reference = "<guid> (<modelo>)"` |
| categoria | `KeywordCategorizer`: palavras-chave em inglês/espanhol/português |

Linhas ignoradas: invisíveis; material perigoso `block` (PVC, neoprene);
modelo sem potência conhecida; ação desconhecida; parâmetros fora de faixa;
duplicatas após normalização.

## Modelos sem potência no nome

Adicione em `tools/lasergrbl/models.toml` **somente com fonte verificável**
(especificação do fabricante, em comentário), depois regenere. Modelos ainda
sem mapeamento estão no `REPORT.md` (ver também `doc/roadmap.md`).

## Outras fontes

Antes de importar qualquer base de terceiros, confirme uma licença explícita
compatível com GPL-3.0 (`validate/sources.rs` mantém a allow-list). Não copie
bibliotecas proprietárias (LightBurn, fabricantes) sem licença explícita.
Para uma fonte nova, crie `import/<fonte>/` com `impl Importer`.
