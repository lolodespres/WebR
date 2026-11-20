//! Le lexer lui-même : on parcourt le texte caractère par caractère et on produit
//! des jetons.
//!
//! ## Les retours à la ligne
//!
//! webRu n'a pas de `;` : un retour à la ligne termine l'instruction. Sauf quand
//! la ligne est visiblement incomplète. Un `\n` est **ignoré** si :
//! 1. on est entre `( )` ou `[ ]` (mais pas entre `{ }`, qui contient des instructions) ;
//! 2. le jeton précédent est une « continuation » (opérateur, virgule…), voir
//!    [`TokenKind::is_continuation`] ;
//! 3. la ligne suivante commence par `->`, `?->`, `??`, `.`, `&&` ou `||`.

use std::fmt;

use crate::token::{Span, StrPart, Token, TokenKind, keyword};

/// Erreur de découpage : message + position.
#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ligne {}, colonne {} : {}",
            self.span.line, self.span.col, self.message
        )
    }
}

impl std::error::Error for LexError {}

/// Découpe `source` en jetons. Le dernier jeton est toujours `Eof`.
pub fn tokenize(source: &str) -> Result<Vec<Token>, LexError> {
    Lexer::new(source, Span { line: 1, col: 1 }).run()
}

/// Peut commencer un nom (de variable, de fonction…) : lettre ou `_`.
fn is_ident_start(c: char) -> bool {
    c == '_' || c.is_alphabetic()
}

/// Peut continuer un nom : lettre, chiffre ou `_`.
fn is_ident_continue(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}

struct Lexer {
    /// Le source, découpé en caractères Unicode (`char`).
    chars: Vec<char>,
    /// Index du prochain caractère à lire dans `chars`.
    pos: usize,
    line: usize,
    col: usize,
    /// Jetons déjà produits.
    tokens: Vec<Token>,
    /// Pile des délimiteurs ouverts : `(`, `[` ou `{`.
    delimiters: Vec<char>,
}

impl Lexer {
    fn new(source: &str, start: Span) -> Self {
        Lexer {
            chars: source.chars().collect(),
            pos: 0,
            line: start.line,
            col: start.col,
            tokens: Vec::new(),
            delimiters: Vec::new(),
        }
    }

    // --- Outils de base ---------------------------------------------------------

    /// Le caractère courant, sans avancer. `None` en fin de fichier.
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    /// Le caractère d'après, sans avancer.
    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    /// Consomme le caractère courant et le renvoie, en mettant à jour ligne/colonne.
    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?; // `?` sur une Option : renvoie `None` tout de suite si vide
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    /// Si le caractère courant est `expected`, le consomme et renvoie `true`.
    fn eat(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Position actuelle (celle du prochain caractère).
    fn span(&self) -> Span {
        Span {
            line: self.line,
            col: self.col,
        }
    }

    /// Fabrique une erreur. Générique sur `T` pour pouvoir écrire
    /// `return self.error(span, "...")` dans n'importe quelle fonction qui renvoie un `Result`.
    fn error<T>(&self, span: Span, message: impl Into<String>) -> Result<T, LexError> {
        Err(LexError {
            message: message.into(),
            span,
        })
    }

    fn push(&mut self, kind: TokenKind, span: Span) {
        match &kind {
            TokenKind::LParen => self.delimiters.push('('),
            TokenKind::LBracket => self.delimiters.push('['),
            TokenKind::LBrace => self.delimiters.push('{'),
            TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => {
                // Un `)` sans `(` sera signalé par le parser, pas ici.
                self.delimiters.pop();
            }
            _ => {}
        }
        self.tokens.push(Token { kind, span });
    }

    // --- Boucle principale -------------------------------------------------------

    fn run(mut self) -> Result<Vec<Token>, LexError> {
        loop {
            self.skip_spaces_and_comments()?;
            let span = self.span();
            let Some(c) = self.peek() else {
                break; // fin du fichier
            };
            let kind = match c {
                '\n' => {
                    self.advance();
                    self.newline(span);
                    continue;
                }
                '$' => self.lex_variable()?,
                '0'..='9' => self.lex_number()?,
                '\'' => self.lex_single_quoted()?,
                '"' => self.lex_double_quoted()?,
                c if is_ident_start(c) => {
                    let word = self.read_ident();
                    keyword(&word).unwrap_or(TokenKind::Ident(word))
                }
                _ => self.lex_operator()?,
            };
            self.push(kind, span);
        }
        let span = self.span();
        self.push(TokenKind::Eof, span);
        Ok(self.tokens)
    }

    fn skip_spaces_and_comments(&mut self) -> Result<(), LexError> {
        loop {
            match (self.peek(), self.peek_next()) {
                (Some(' ' | '\t' | '\r'), _) => {
                    self.advance();
                }
                (Some('/'), Some('/')) => {
                    // Commentaire jusqu'à la fin de ligne ; le `\n` reste à traiter.
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.advance();
                    }
                }
                (Some('/'), Some('*')) => {
                    let start = self.span();
                    self.advance();
                    self.advance();
                    loop {
                        match self.advance() {
                            None => return self.error(start, "commentaire `/*` jamais fermé"),
                            Some('*') if self.eat('/') => break,
                            Some(_) => {}
                        }
                    }
                }
                _ => return Ok(()),
            }
        }
    }

    /// Un `\n` vient d'être consommé : faut-il produire un jeton `Newline` ?
    fn newline(&mut self, span: Span) {
        // 1. Entre ( ) ou [ ], les retours à la ligne ne comptent pas.
        if matches!(self.delimiters.last(), Some('(' | '[')) {
            return;
        }
        // 2. Début de fichier, ou ligne précédente terminée par une continuation.
        match self.tokens.last() {
            None => return,
            Some(token) if token.kind.is_continuation() => return,
            Some(_) => {}
        }
        // 3. La ligne suivante commence par un opérateur de chaînage.
        if self.next_line_continues() {
            return;
        }
        self.push(TokenKind::Newline, span);
    }

    /// Regarde (sans consommer) si la prochaine ligne non vide commence par
    /// `->`, `?->`, `??`, `.`, `&&` ou `||`.
    fn next_line_continues(&self) -> bool {
        let rest = &self.chars[self.pos..];
        let start = rest
            .iter()
            .position(|c| !matches!(c, ' ' | '\t' | '\r' | '\n'))
            .unwrap_or(rest.len());
        let rest = &rest[start..];
        matches!(
            rest,
            ['-', '>', ..]
                | ['?', '-', '>', ..]
                | ['?', '?', ..]
                | ['.', ..]
                | ['&', '&', ..]
                | ['|', '|', ..]
        )
    }

    // --- Chaînes de caractères ---------------------------------------------------

    /// `'texte brut'` : pas d'interpolation, seuls `\'` et `\\` sont des échappements.
    fn lex_single_quoted(&mut self) -> Result<TokenKind, LexError> {
        let start = self.span();
        self.advance(); // le `'` ouvrant
        let mut text = String::new();
        loop {
            match self.advance() {
                None => return self.error(start, "chaîne jamais fermée : il manque un `'`"),
                Some('\'') => break,
                Some('\\') if matches!(self.peek(), Some('\'' | '\\')) => {
                    if let Some(c) = self.advance() {
                        text.push(c);
                    }
                }
                Some(c) => text.push(c),
            }
        }
        Ok(TokenKind::Str(vec![StrPart::Lit(text)]))
    }

    /// `"texte $interpolé {$expr}"`
    fn lex_double_quoted(&mut self) -> Result<TokenKind, LexError> {
        let start = self.span();
        self.advance(); // le `"` ouvrant
        let mut parts = Vec::new();
        let mut text = String::new();
        loop {
            let Some(c) = self.advance() else {
                return self.error(start, "chaîne jamais fermée : il manque un `\"`");
            };
            match c {
                '"' => break,
                '\\' => match self.advance() {
                    Some('n') => text.push('\n'),
                    Some('t') => text.push('\t'),
                    Some('r') => text.push('\r'),
                    Some(c @ ('\\' | '"' | '$' | '{')) => text.push(c),
                    // Échappement inconnu : on garde tel quel, comme PHP.
                    Some(c) => {
                        text.push('\\');
                        text.push(c);
                    }
                    None => return self.error(start, "chaîne jamais fermée : il manque un `\"`"),
                },
                '$' if self.peek().is_some_and(is_ident_start) => {
                    // `std::mem::take` vide `text` et nous rend son ancien contenu.
                    if !text.is_empty() {
                        parts.push(StrPart::Lit(std::mem::take(&mut text)));
                    }
                    parts.push(StrPart::Var(self.read_ident()));
                }
                '{' if self.peek() == Some('$') => {
                    if !text.is_empty() {
                        parts.push(StrPart::Lit(std::mem::take(&mut text)));
                    }
                    let code_start = self.span();
                    let code = self.read_interpolated_code(start)?;
                    // Un second lexer, sur le seul code entre `{` et `}`.
                    let tokens = Lexer::new(&code, code_start).run()?;
                    parts.push(StrPart::Expr(tokens));
                }
                c => text.push(c),
            }
        }
        if !text.is_empty() || parts.is_empty() {
            parts.push(StrPart::Lit(text));
        }
        Ok(TokenKind::Str(parts))
    }

    /// Lit le code d'une interpolation `{$...}` jusqu'à l'accolade fermante
    /// correspondante (le `{` ouvrant a déjà été consommé). Gère les accolades et les
    /// chaînes imbriquées : `"{$a['}']}"`.
    fn read_interpolated_code(&mut self, string_start: Span) -> Result<String, LexError> {
        let mut code = String::new();
        let mut depth = 1;
        let mut quote: Option<char> = None;
        loop {
            let Some(c) = self.advance() else {
                return self.error(string_start, "interpolation `{$...}` jamais fermée");
            };
            match quote {
                Some(q) => {
                    if c == '\\' {
                        code.push(c);
                        if let Some(escaped) = self.advance() {
                            code.push(escaped);
                        }
                        continue;
                    }
                    if c == q {
                        quote = None;
                    }
                }
                None => match c {
                    '\'' | '"' => quote = Some(c),
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            return Ok(code);
                        }
                    }
                    _ => {}
                },
            }
            code.push(c);
        }
    }

    /// Lit un nom (`[lettre|_|chiffre]*`) à partir de la position courante et le renvoie.
    /// Peut renvoyer une chaîne vide si le caractère courant ne peut pas continuer un nom.
    fn read_ident(&mut self) -> String {
        let mut name = String::new();
        while let Some(c) = self.peek() {
            if !is_ident_continue(c) {
                break;
            }
            name.push(c);
            self.advance();
        }
        name
    }

    /// Lit une variable : `$nom` → `TokenKind::Variable("nom")`.
    /// Le caractère courant est le `$`.
    ///
    /// Erreur si le `$` n'est pas suivi d'un caractère qui peut **commencer** un nom
    /// (`is_ident_start`) : `$`, `$1`, `$ x`…
    fn lex_variable(&mut self) -> Result<TokenKind, LexError> {
        let span = self.span();
        self.advance();
        let debut_valide = match self.peek() {
            Some(c) => is_ident_start(c),
            None => false,
        };
        if !debut_valide {
            return self.error(span, "Invalid variable name");
        }
        Ok(TokenKind::Variable(self.read_ident()))
    }

    /// Lit un nombre. Le caractère courant est un chiffre.
    ///
    /// - `42` → `Int(42)` ; `1_000_000` → `Int(1000000)` (les `_` sont ignorés) ;
    /// - `3.14` → `Float(3.14)` : c'est un flottant seulement si le `.` est **suivi d'un
    ///   chiffre** (`1.` suivi d'autre chose reste `Int(1)` puis `Dot`) ;
    /// - un entier qui ne tient pas dans un `i64` → erreur (pas de panique !).
    fn lex_number(&mut self) -> Result<TokenKind, LexError> {
        todo!("lex_number")
    }

    /// Lit un opérateur ou un délimiteur. Le caractère courant n'est ni un espace,
    /// ni un chiffre, ni une lettre, ni `$`, ni un guillemet.
    ///
    /// Opérateurs reconnus : voir les variantes de [`TokenKind`]. Toujours prendre l'opérateur
    /// **le plus long** possible : `+=` et pas `+` puis `=`. `&`, `|` et `?` seuls n'existent pas.
    /// Le `;` donne une erreur dont le message contient « point-virgule ». Tout autre caractère
    /// inconnu : erreur aussi.
    fn lex_operator(&mut self) -> Result<TokenKind, LexError> {
        todo!("lex_operator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use TokenKind as K;

    /// Raccourci : découpe `src` et ne garde que la nature des jetons.
    fn kinds(src: &str) -> Vec<TokenKind> {
        tokenize(src)
            .expect("le lexer n'aurait pas dû échouer")
            .into_iter()
            .map(|t| t.kind)
            .collect()
    }

    /// Raccourci : découpe `src` en s'attendant à une erreur.
    fn lex_err(src: &str) -> LexError {
        tokenize(src).expect_err("une erreur était attendue")
    }

    fn var(name: &str) -> TokenKind {
        K::Variable(name.to_string())
    }

    fn lit(text: &str) -> StrPart {
        StrPart::Lit(text.to_string())
    }

    // --- Noms et variables ----------------------------------------------------------

    #[test]
    fn variables() {
        assert_eq!(kinds("$nom"), vec![var("nom"), K::Eof]);
        assert_eq!(kinds("$_x1"), vec![var("_x1"), K::Eof]);
        assert_eq!(kinds("$visité"), vec![var("visité"), K::Eof]);
        assert_eq!(kinds("$a $b"), vec![var("a"), var("b"), K::Eof]);
    }

    #[test]
    fn variable_dollar_sans_nom() {
        assert_eq!(lex_err("$").span, Span { line: 1, col: 1 });
        assert_eq!(lex_err("$1").span, Span { line: 1, col: 1 });
        assert_eq!(lex_err("$a $ b").span, Span { line: 1, col: 4 });
    }

    #[test]
    fn identifiants_et_mots_cles() {
        assert_eq!(
            kinds("fn compte echo None"),
            vec![
                K::Fn,
                K::Ident("compte".to_string()),
                K::Echo,
                K::NoneKw,
                K::Eof
            ]
        );
    }

    // --- Nombres --------------------------------------------------------------------

    #[test]
    fn nombres_entiers() {
        assert_eq!(kinds("0"), vec![K::Int(0), K::Eof]);
        assert_eq!(kinds("42"), vec![K::Int(42), K::Eof]);
        assert_eq!(kinds("1_000_000"), vec![K::Int(1_000_000), K::Eof]);
        assert_eq!(kinds("9223372036854775807"), vec![K::Int(i64::MAX), K::Eof]);
    }

    #[test]
    fn nombres_flottants() {
        assert_eq!(kinds("2.75"), vec![K::Float(2.75), K::Eof]);
        assert_eq!(kinds("0.5"), vec![K::Float(0.5), K::Eof]);
        assert_eq!(kinds("1_000.25"), vec![K::Float(1000.25), K::Eof]);
    }

    #[test]
    fn nombres_point_sans_chiffre_apres() {
        assert_eq!(
            kinds("1.x"),
            vec![K::Int(1), K::Dot, K::Ident("x".to_string()), K::Eof]
        );
    }

    #[test]
    fn nombres_entier_trop_grand() {
        let err = lex_err("9223372036854775808");
        assert_eq!(err.span, Span { line: 1, col: 1 });
    }

    // --- Opérateurs -----------------------------------------------------------------

    #[test]
    fn operateurs_simples() {
        assert_eq!(
            kinds("( ) { } [ ] , : + - * / % . = ! < >"),
            vec![
                K::LParen,
                K::RParen,
                K::LBrace,
                K::RBrace,
                K::LBracket,
                K::RBracket,
                K::Comma,
                K::Colon,
                K::Plus,
                K::Minus,
                K::Star,
                K::Slash,
                K::Percent,
                K::Dot,
                K::Assign,
                K::Bang,
                K::Lt,
                K::Gt,
                K::Eof
            ]
        );
    }

    #[test]
    fn operateurs_composes() {
        assert_eq!(
            kinds("+= -= .= == != <= >= && || ?? -> ?-> => ::"),
            vec![
                K::PlusAssign,
                K::MinusAssign,
                K::DotAssign,
                K::EqEq,
                K::NotEq,
                K::LtEq,
                K::GtEq,
                K::AndAnd,
                K::OrOr,
                K::QuestionQuestion,
                K::Arrow,
                K::QuestionArrow,
                K::FatArrow,
                K::ColonColon,
                K::Eof
            ]
        );
    }

    #[test]
    fn operateurs_colles() {
        assert_eq!(
            kinds("$a+=1"),
            vec![var("a"), K::PlusAssign, K::Int(1), K::Eof]
        );
        assert_eq!(
            kinds("$u?->nom"),
            vec![
                var("u"),
                K::QuestionArrow,
                K::Ident("nom".to_string()),
                K::Eof
            ]
        );
        assert_eq!(kinds("$a-1"), vec![var("a"), K::Minus, K::Int(1), K::Eof]);
    }

    #[test]
    fn operateurs_point_virgule_interdit() {
        let err = lex_err("+;");
        assert_eq!(err.span, Span { line: 1, col: 2 });
        assert!(
            err.message.contains("point-virgule"),
            "message reçu : {}",
            err.message
        );
    }

    #[test]
    fn operateurs_caracteres_inconnus() {
        assert_eq!(lex_err("@").span, Span { line: 1, col: 1 });
        assert_eq!(lex_err("+ &").span, Span { line: 1, col: 3 }); // `&` seul n'existe pas
        assert_eq!(lex_err("?").span, Span { line: 1, col: 1 }); // `?` seul non plus
    }

    // --- Chaînes --------------------------------------------------------------------

    #[test]
    fn chaine_apostrophes() {
        assert_eq!(kinds(r"'l\'été'"), vec![K::Str(vec![lit("l'été")]), K::Eof]);
        assert_eq!(
            kinds("'pas de $interpolation'"),
            vec![K::Str(vec![lit("pas de $interpolation")]), K::Eof]
        );
        assert_eq!(kinds("''"), vec![K::Str(vec![lit("")]), K::Eof]);
    }

    #[test]
    fn chaine_echappements() {
        assert_eq!(
            kinds(r#""a\tb\n\"c\" \$x \{""#),
            vec![K::Str(vec![lit("a\tb\n\"c\" $x {")]), K::Eof]
        );
    }

    #[test]
    fn chaine_interpolation_variable() {
        assert_eq!(
            kinds(r#""Bonjour $nom !""#),
            vec![
                K::Str(vec![
                    lit("Bonjour "),
                    StrPart::Var("nom".to_string()),
                    lit(" !")
                ]),
                K::Eof
            ]
        );
        // `$` non suivi d'un nom, `{` non suivi de `$` : texte normal.
        assert_eq!(
            kinds(r#""5 $ {json}""#),
            vec![K::Str(vec![lit("5 $ {json}")]), K::Eof]
        );
    }

    #[test]
    fn chaine_interpolation_expression() {
        let tokens = kinds(r#""visite n° {$session['visites']} !""#);
        let K::Str(parts) = &tokens[0] else {
            panic!("une chaîne était attendue, reçu {:?}", tokens[0]);
        };
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], lit("visite n° "));
        assert_eq!(parts[2], lit(" !"));
        let StrPart::Expr(inner) = &parts[1] else {
            panic!("une expression était attendue, reçu {:?}", parts[1]);
        };
        let inner_kinds: Vec<TokenKind> = inner.iter().map(|t| t.kind.clone()).collect();
        assert_eq!(
            inner_kinds,
            vec![
                var("session"),
                K::LBracket,
                K::Str(vec![lit("visites")]),
                K::RBracket,
                K::Eof
            ]
        );
        // Les positions des jetons internes pointent dans le fichier d'origine.
        assert_eq!(inner[0].span, Span { line: 1, col: 13 });
    }

    #[test]
    fn chaine_non_fermee() {
        assert_eq!(lex_err("'abc").span, Span { line: 1, col: 1 });
        assert_eq!(lex_err("$a = \"abc").span, Span { line: 1, col: 6 });
    }

    // --- Retours à la ligne ---------------------------------------------------------

    #[test]
    fn retour_ligne_termine_instruction() {
        assert_eq!(
            kinds("$a = 1\n$b = 2"),
            vec![
                var("a"),
                K::Assign,
                K::Int(1),
                K::Newline,
                var("b"),
                K::Assign,
                K::Int(2),
                K::Eof
            ]
        );
    }

    #[test]
    fn retour_ligne_lignes_vides_fusionnees() {
        assert_eq!(
            kinds("\n\n$a\n\n\n$b\n"),
            vec![var("a"), K::Newline, var("b"), K::Newline, K::Eof]
        );
    }

    #[test]
    fn retour_ligne_apres_operateur() {
        assert_eq!(
            kinds("$a = 1 +\n    2"),
            vec![var("a"), K::Assign, K::Int(1), K::Plus, K::Int(2), K::Eof]
        );
    }

    #[test]
    fn retour_ligne_entre_parentheses() {
        assert_eq!(
            kinds("f(1\n, 2\n)"),
            vec![
                K::Ident("f".to_string()),
                K::LParen,
                K::Int(1),
                K::Comma,
                K::Int(2),
                K::RParen,
                K::Eof
            ]
        );
    }

    #[test]
    fn retour_ligne_operateur_en_debut_de_ligne() {
        assert_eq!(
            kinds("$s = 'a'\n    . 'b'"),
            vec![
                var("s"),
                K::Assign,
                K::Str(vec![lit("a")]),
                K::Dot,
                K::Str(vec![lit("b")]),
                K::Eof
            ]
        );
    }

    #[test]
    fn retour_ligne_dans_un_bloc() {
        assert_eq!(
            kinds("if $ok {\n    echo 1\n}"),
            vec![
                K::If,
                var("ok"),
                K::LBrace,
                K::Echo,
                K::Int(1),
                K::Newline,
                K::RBrace,
                K::Eof
            ]
        );
    }

    #[test]
    fn commentaires() {
        assert_eq!(
            kinds("$a // fin de ligne\n$b /* bloc\nsur deux lignes */ $c"),
            vec![var("a"), K::Newline, var("b"), var("c"), K::Eof]
        );
        assert_eq!(lex_err("/* jamais fermé").span, Span { line: 1, col: 1 });
    }

    #[test]
    fn positions() {
        let tokens = tokenize("$a = 1\n  $b").unwrap();
        assert_eq!(tokens[0].span, Span { line: 1, col: 1 });
        assert_eq!(tokens[2].span, Span { line: 1, col: 6 });
        assert_eq!(tokens[4].span, Span { line: 2, col: 3 });
    }
}
