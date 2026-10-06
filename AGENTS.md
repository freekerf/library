# AGENTS.md — FreeKerf Library

Guia de manutenção para pessoas e agentes (Claude, Codex, Copilot…). Leia
antes de mudar qualquer coisa. Este arquivo faz parte da **documentação viva**:
se a sua mudança o deixa desatualizado, atualize-o no mesmo PR.

## Visão geral

Biblioteca aberta de **materiais** (receitas de corte/gravação) e **perfis de
máquina** do FreeKerf, consumida pelo crate `freekerf-materials` do monorepo
[`freekerf/freekerf`](https://github.com/freekerf/freekerf) e por qualquer
outro software via um pacote versionado (`library-vX.Y.Z.tar.zst` + índice JSON).

O repositório tem duas partes:

| Parte | Onde | O quê |
| --- | --- | --- |
| Dados | `data/` | TOML legível em diff: materiais, máquinas, lista de perigosos |
| Ferramentas | `crates/` | Rust: modelo, schema, validação, empacotamento, importadores |

```
data/materials/<categoria>/<id>.toml     materiais curados
data/materials/imported/lasergrbl/       importação gerada do LaserGRBL (não editar à mão)
data/machines/<fabricante>/<id>.toml     perfis de máquina
data/safety/hazards.toml                 materiais perigosos (block/warn)
schema/v1/*.schema.json                  JSON Schema GERADO a partir do modelo Rust
crates/freekerf-library/                 biblioteca (modelo, loader, regras, pacote, import)
crates/freekerf-library-cli/             binário `freekerf-library`
tools/lasergrbl/                         mapa de modelos + cópia do arquivo upstream
evidence/                                fotos de grades de teste (receitas `tested`)
doc/architecture.md                      arquitetura
doc/features/                            uma página por funcionalidade
doc/roadmap.md                           o que ficou para depois e decisões em aberto
```

## Comandos

```sh
cargo run -- validate                    # schema + lint TOML + regras semânticas
cargo run -- validate --deny-warnings    # inclui avisos
cargo run -- schema                      # regenera schema/v1 a partir do modelo
cargo run -- schema --check              # CI: schema commitado == gerado
cargo run -- package --version 0.1.0     # dist/library-v0.1.0.tar.zst + .index.json
cargo run -- import lasergrbl --clean \
  --input tools/lasergrbl/upstream/StandardMaterials.psh
cargo test                               # unitários + integração
cargo llvm-cov --workspace --summary-only   # cobertura (CI exige ≥ 90 % de linhas)
cargo fmt --all && cargo clippy --all-targets -- -D warnings
```

## Arquitetura em uma tela

```
data/*.toml ─► Loader ─(DocumentCheck: TextLint, SchemaCheck)─► Library
                                                            │
                                   Validator ─(Rule × N)────┤─► Diagnostics
                                                            │
                                                package::build ─► .tar.zst + index.json
LaserGRBL .psh ─► LaserGrblImporter(LaserResolver, Categorizer, HazardList) ─► TOML
```

Detalhes em [`doc/architecture.md`](doc/architecture.md).

## Boas práticas: SOLID aplicado aqui

- **S — responsabilidade única.** Cada regra de validação é um tipo em
  `validate/<regra>.rs` que verifica uma coisa. O loader só carrega; o
  validador só valida; o empacotador só empacota; a CLI só liga a biblioteca
  ao terminal (sem lógica de negócio em `crates/freekerf-library-cli`).
- **O — aberto/fechado.** Nova verificação = novo `impl Rule` (semântica) ou
  `impl DocumentCheck` (arquivo cru), registrado em `Validator::standard()` /
  `Loader::default()`. Não edite regras existentes para acomodar casos novos.
- **L — substituição de Liskov.** Qualquer `Rule`, `DocumentCheck`,
  `LaserResolver`, `Categorizer` ou `Importer` deve poder trocar outro sem
  surpresas: não entre em pânico com dados válidos, só registre diagnósticos.
- **I — segregação de interfaces.** Traits pequenos (um método essencial).
  Não acrescente métodos "opcionais" a um trait existente; crie outro.
- **D — inversão de dependências.** O importador recebe `&dyn LaserResolver`,
  `&dyn Categorizer` e a `HazardList` por injeção; testes usam implementações
  próprias. Siga o mesmo padrão em importadores novos.

## Regras de dados (invioláveis)

1. **Unidades explícitas** no nome do campo: `_mm_min`, `_pct`, `_mm`, `_w`, `dpi`.
2. **Nunca** marque uma receita como `tested` sem foto da grade de teste e autor;
   valores escritos de cabeça são `estimated` (com `method`). O app nunca
   apresenta `estimated` como garantido.
3. **Nunca** importe dados de terceiros sem licença explícita compatível com a
   GPL-3.0 (`source.license` passa pela allow-list em `validate/sources.rs`).
   Bibliotecas do LightBurn ou de fabricantes estão fora, salvo licença explícita.
4. Materiais que casam com um perigo `block` são rejeitados pela validação.
5. `data/materials/imported/**` é gerado: mude o importador ou
   `tools/lasergrbl/models.toml` e regenere; não edite os TOML à mão.
   O teste `lasergrbl_import_is_reproducible` garante isso.
6. Schema v1 é contrato público: mudanças incompatíveis vão para `schema/v2`
   (e `FORMAT_VERSION` do índice). Campos opcionais novos são compatíveis.

## Documentação viva

Toda mudança de comportamento atualiza a documentação **no mesmo PR**:

| Mudou… | Atualize |
| --- | --- |
| modelo (`model/`) | `cargo run -- schema`, `doc/features/data-format.md` |
| regra de validação | `doc/features/validation.md` |
| perigos / segurança | `doc/features/safety.md` |
| importador | `doc/features/import-lasergrbl.md` (e regenere os dados) |
| pacote / índice | `doc/features/packaging.md` |
| módulos, traits, fluxo | `doc/architecture.md` e a seção "Arquitetura" acima |
| algo adiado | `doc/roadmap.md` |

Os schemas em `schema/v1/` são documentação gerada a partir do código (doc
comments viram `description`), e os testes de integração verificam que dados,
schemas e importação commitados batem com o código.

## Testes

- Unitários ficam junto do código (`#[cfg(test)] mod tests`); helpers de
  regras em `validate::fixtures`.
- Integração: `crates/freekerf-library/tests/repository.rs` (dados reais,
  schemas, importação reproduzível, pacote ida-e-volta) e
  `crates/freekerf-library-cli/tests/cli.rs` (binário ponta a ponta).
- Cobertura medida com `cargo llvm-cov`; o CI falha abaixo de 90 % de linhas.
  Código novo vem com teste.

## Convenções

- Documentação e mensagens para contribuidores em **português**; código,
  identificadores, chaves e valores dos dados em **inglês** (interoperabilidade).
- Rust edition 2024, `cargo fmt` (largura 110), `clippy -D warnings`,
  `#![forbid(unsafe_code)]` via lints do workspace, `missing_docs` ligado.
- Saídas determinísticas (ordenação estável, pacote reproduzível).
- Licença: GPL-3.0-or-later (ver `NOTICE`). Sem telemetria (decisão do projeto).
