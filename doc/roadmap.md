# Roadmap e decisões em aberto

## Decidido

- **Telemetria**: não haverá.
- **Linguagem das ferramentas**: Rust.

## Próximos passos

- **Licença por submissão** (não implementado): os dados poderão ter licença
  própria, com padrão **CC BY-SA 4.0**, e quem submete poderá escolher a licença
  da sua contribuição. Exige um campo de licença por material/receita, regras
  de compatibilidade e um pacote que respeite licenças mistas. Atenção: dados
  derivados do LaserGRBL são GPL-3.0 e não podem ser relicenciados para
  CC BY-SA sem autorização dos autores originais. Até lá, tudo segue
  GPL-3.0-or-later.
- **Curadoria da importação LaserGRBL**: mapear os modelos ainda sem potência
  conhecida (ver `data/materials/imported/lasergrbl/REPORT.md`), unificar nomes
  em espanhol/inglês, informar espessura dos cortes (avisos atuais da validação).
- **Grades de teste** para promover receitas curadas de `estimated` a `tested`.
- **Parâmetros de fibra** (frequência, largura de pulso) — fora do schema v1.
- **i18n** de nomes e motivos de perigo (hoje em inglês).
- **Perfis de máquina reais** de fabricantes, com fonte.
- Integração com `freekerf-materials` (marco M2 do monorepo) consumindo a
  primeira release.
