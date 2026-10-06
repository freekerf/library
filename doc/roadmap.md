# Roadmap e decisões em aberto

## Decidido

- **Telemetria**: não haverá.
- **Linguagem das ferramentas**: Rust.
- **Consumo pelo `freekerf-materials`**: via dependência git deste crate
  (`publish = false`, sem publicação no crates.io por enquanto).
- **Perfis de máquina**: a primeira release só traz perfis genéricos de referência.
- **Modelos do LaserGRBL sem potência conhecida**: ficam de fora até termos
  uma fonte verificável.

## Próximos passos

- **Licença por submissão** (não implementado): licenças mistas no mesmo
  pacote, decididas assim:
  - dados importados do LaserGRBL (e derivados deles) continuam
    **GPL-3.0-or-later**;
  - contribuições novas usam **CC BY-SA 4.0** por padrão, e quem submete pode
    escolher outra licença compatível;
  - cada material ou receita declara a sua licença, e o pacote é um agregado
    de arquivos com licenças diferentes, cada um com a sua atribuição;
  - a compatibilidade só vale num sentido: um dado CC BY-SA 4.0 pode ser
    adaptado para dentro de algo GPL-3.0, mas um dado GPL não vira
    CC BY-SA. Uma receita derivada de dado GPL (por exemplo, `estimated` com
    `derived_from` apontando para um dado GPL) continua GPL.
  Para implementar: campo `license` (SPDX) por material/receita, uma regra de
  validação que confira a direção da compatibilidade em `derived_from`, e
  `license` por entrada no `index.json`. Até lá, tudo segue GPL-3.0-or-later.
- **Curadoria da importação LaserGRBL**: mapear os modelos ainda sem potência
  conhecida (ver `data/materials/imported/lasergrbl/REPORT.md`), unificar nomes
  em espanhol/inglês, informar espessura dos cortes (avisos atuais da validação).
- **Grades de teste** para promover receitas curadas de `estimated` a `tested`.
- **Parâmetros de fibra** (frequência, largura de pulso) — fora do schema v1.
- **i18n** de nomes e motivos de perigo (hoje em inglês).
- **Perfis de máquina reais** de fabricantes, com fonte (depois da primeira release).
- Integração com `freekerf-materials` (marco M2 do monorepo) consumindo a
  primeira release.
