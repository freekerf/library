# Segurança: materiais perigosos

Arquivo: `data/safety/hazards.toml` (schema `hazards.schema.json`).

```toml
[[hazards]]
id = "pvc"
name = "PVC / vinyl"
severity = "block"            # block = nunca processar; warn = alertar
keywords = ["pvc", "vinyl", "vinil"]
unless = []                   # frases que anulam o alerta (ex.: "vegetable tanned")
reason = "Releases hydrogen chloride…"
emissions = ["hydrogen chloride"]
```

- Casamento por **palavra inteira**, sem diferenciar maiúsculas nem acentos
  (`Poly_Vinyl_Chloride` casa com `vinyl`); palavras-chave precisam estar
  normalizadas (minúsculas, ASCII, separadas por espaço) — a validação confere.
- `block`: PVC/vinil, policarbonato, fibra de vidro/carbono, couro curtido ao
  cromo, PTFE, neoprene. Um material com esses nomes **reprova** a validação e
  o importador descarta essas linhas.
- `warn`: couro de curtimento desconhecido (exceto "vegetable tanned"), couro
  sintético, ABS, espumas (exceto EVA), HDPE, borracha, galvanizado.
- O pacote exporta os perigos de cada material em `index.json`
  (`materials[].hazards`), para o app **bloquear** (`block`) ou **alertar**
  (`warn`) sem refazer o casamento. A lista completa vai em `hazards.json`
  para o app checar também nomes digitados pelo usuário.
