# Como contribuir com uma receita

1. Crie/edite `data/materials/<categoria>/<id>.toml` (formato em
   [data-format.md](data-format.md)). Para uma receita `tested`, inclua a foto
   da grade de teste em `evidence/<material-id>/`.
2. `cargo run -- validate` até zerar os erros.
3. Abra o PR com o template de receita (`?template=new-recipe.md` na URL).
4. O CI valida schema, unidades, faixas plausíveis, chaves únicas, lint de
   TOML, licença da origem e a lista de perigos.

Sem foto + autor, a receita entra como `community` (relato) ou `estimated`
(derivada), nunca como `tested`.
