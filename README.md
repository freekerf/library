# FreeKerf Library

Materiais (parâmetros de corte e gravação) e perfis de máquina abertos para o
[FreeKerf](https://github.com/freekerf/freekerf), validados por código e
publicados como um pacote versionado (`library-vX.Y.Z.tar.zst` + índice JSON)
que qualquer software pode usar.

- **Dados**: `data/` (TOML) — 10 materiais curados, a base do LaserGRBL importada,
  perfis de máquina de referência e a lista de materiais perigosos.
- **Formato**: [`schema/v1/`](schema/v1) (JSON Schema gerado do modelo Rust) —
  ver [doc/features/data-format.md](doc/features/data-format.md).
- **Ferramentas**: `cargo run -- validate | schema | package | import lasergrbl`.
- **Contribuir**: [doc/features/contributing.md](doc/features/contributing.md).
- **Manutenção**: [AGENTS.md](AGENTS.md), [arquitetura](doc/architecture.md),
  [funcionalidades](doc/features/README.md), [roadmap](doc/roadmap.md).

Licença: [GPL-3.0-or-later](LICENSE). Derivado do LaserGRBL, Copyright (c) 2016
Diego Settimi ([NOTICE](NOTICE)); os dados importados do LaserGRBL mantêm a
atribuição em `source` de cada receita.
