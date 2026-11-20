//! # webru-lexer
//!
//! Première étape du moteur webRu : découper le texte source en **jetons** (tokens).
//!
//! ```text
//! $nom = "Léa"   →   Variable("nom")  Assign  Str([Lit("Léa")])  Eof
//! ```
//!
//! Particularités de webRu gérées ici :
//! - les variables commencent par `$` ;
//! - pas de point-virgule : le retour à la ligne termine l'instruction, sauf quand la
//!   ligne est visiblement incomplète (voir le module `lexer`) ;
//! - l'interpolation dans les chaînes : `"Bonjour $nom, {$user['age']} ans"`.

mod lexer;
mod token;

pub use lexer::{LexError, tokenize};
pub use token::{Span, StrPart, Token, TokenKind, keyword};
