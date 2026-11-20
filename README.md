# webRu

Un langage web inspiré de PHP (le `$`, les tableaux associatifs, un cycle de vie par requête),
sans point-virgule et sans `null` (`Option<T>` à la place). Le moteur est écrit en Rust.

Projet personnel et fil rouge pour apprendre Rust. Vision complète :
[docs/cahier-des-charges.md](docs/cahier-des-charges.md).

## État

| Étape | État |
|---|---|
| Initialisation (dépôt, workspace Cargo) | fait |
| Lexer | en cours |
| Parser (AST) | à faire |
| Interpréteur v0.1 | à faire |
| Intégration web (`axum`, `$session`) | à faire |
| Bibliothèque standard, éditions | à faire |

## Organisation

```text
crates/
  webru-lexer/     texte source -> jetons
  webru-parser/    jetons -> AST
  webru-runtime/   exécution de l'AST, arène mémoire, sessions
  webru-cli/       la commande `webru`
docs/              cahier des charges, décisions de conception
examples/          scripts webRu (.wru)
```

## Commandes

```sh
cargo test                                   # tous les tests
cargo test -p webru-lexer nombres            # les tests dont le nom contient « nombres »
cargo webru tokens examples/bienvenue.wru    # affiche les jetons d'un script
cargo clippy --all-targets                   # linter
cargo fmt                                    # formatage standard
```

## Licence

GPL v3 : voir [LICENSE](LICENSE).
