# Pacote de release

Uma tag `vX.Y.Z` dispara `.github/workflows/release.yml`, que valida e publica
na release do GitHub:

- `library-vX.Y.Z.tar.zst` — o pacote;
- `library-vX.Y.Z.index.json` — cópia do índice (para o app decidir se baixa);
- `SHA256SUMS`.

Local: `cargo run -- package --version X.Y.Z [--out dist]` (recusa empacotar
se houver erro de validação).

## Conteúdo

```
library-vX.Y.Z/
  index.json                 índice (schema/v1/index.schema.json)
  materials/<id>.json        um material por arquivo (mesmo modelo do TOML)
  machines/<id>.json
  hazards.json
  schema/v1/*.schema.json
  LICENSE, NOTICE
```

`index.json`: `format = "freekerf-library"`, `format_version = 1`, `version`,
`schema = "v1"`, `license`, e para cada material `id, name, category,
thickness_mm, operations, lasers, confidence, hazards` + `file { path, sha256 }`;
para cada máquina `id, manufacturer, model, laser, file`; `hazards` e `extra`
(schemas, licença) com SHA-256.

Reprodutível: mesma entrada → mesmos bytes (entradas ordenadas, mtime de
`SOURCE_DATE_EPOCH` — no CI, a data do commit — dono 0, zstd nível 19).

## Consumir

Em Rust (ex.: `freekerf-materials`), dependa do crate `freekerf-library` por git:

```rust
let package = freekerf_library::package::read_archive(path)?; // verifica SHA-256
let materials = package.materials()?;
```

Em outras linguagens: descompacte (zstd + tar), leia `index.json`, confira os
SHA-256 e carregue os JSON (valide com os schemas incluídos).
Versionamento semver: patch = dados corrigidos; minor = dados/campos opcionais
novos; major = mudança incompatível (novo `schema/vN` e `format_version`).
