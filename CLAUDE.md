# webRu — consignes pour Claude

Langage web inspiré de PHP, écrit en Rust. Vision : `docs/cahier-des-charges.md`. Choix de
langage déjà tranchés : `docs/decisions.md` (à consulter avant de proposer une syntaxe).

Dépôt : https://github.com/lolodespres/WebR (public, licence GPL v3). L'identité git est
configurée localement à ce dépôt (adresse noreply GitHub). C'est l'utilisateur qui pousse.

## Toolchain

Rust stable récent (1.100 au 2026-11-20), **édition 2024**. Code formaté avec `cargo fmt`,
clippy (`cargo clippy --all-targets`) sans avertissement.

## Conventions

- Identifiants en anglais ; commentaires, docs et messages d'erreur du langage en français.
- Dépendances externes : seulement quand elles se justifient. Le lexer est écrit à la main
  (plutôt que `logos`) parce que l'interpolation `{$...}` s'y prête mal.
- Tests nommés par thème (`nombres_*`, `operateurs_*`…).
- Scripts webRu : extension `.wru`.
