//! # webru-parser
//!
//! Deuxième étape du moteur : transformer la suite de jetons produite par
//! `webru-lexer` en **arbre syntaxique abstrait** (AST).
//!
//! Prévu pour la v0.1 : expressions avec priorités d'opérateurs, instructions
//! (`echo`, affectation, `if`, `while`, `foreach`), fonctions typées, `Option<T>`.
