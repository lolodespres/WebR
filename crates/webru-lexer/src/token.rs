//! Définition des jetons (tokens) produits par le lexer.

/// Position d'un jeton dans le fichier source (ligne et colonne, comptées à partir de 1).
///
/// `Copy` : une `Span` est petite (deux entiers), on la copie au lieu de la « déplacer ».
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

/// Un jeton : sa nature (`kind`) et l'endroit où il commence (`span`).
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

/// Un morceau de chaîne de caractères.
///
/// `"Bonjour $nom, tu as {$user['age']} ans"` donne :
/// `[Lit("Bonjour "), Var("nom"), Lit(", tu as "), Expr([...]), Lit(" ans")]`
#[derive(Debug, Clone, PartialEq)]
pub enum StrPart {
    /// Texte brut.
    Lit(String),
    /// Variable interpolée : `"$nom"` (sans le `$`).
    Var(String),
    /// Expression interpolée `{$...}`, déjà découpée en jetons (le parser s'en chargera).
    Expr(Vec<Token>),
}

/// La nature d'un jeton.
///
/// Contrairement aux enums PHP, chaque variante peut porter ses propres données :
/// `Int` porte un `i64`, `Variable` un `String`, `Plus` rien du tout.
///
/// Pas de `Eq` ici, seulement `PartialEq` : à cause de `Float(f64)`.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // --- Littéraux et noms -------------------------------------------------
    /// `42`, `1_000`
    Int(i64),
    /// `3.14`
    Float(f64),
    /// `'texte'` ou `"texte $interpolé"`
    Str(Vec<StrPart>),
    /// `$nom` (le nom est stocké sans le `$`)
    Variable(String),
    /// Nom nu : fonction, type… (`count`, `int`, `Option`)
    Ident(String),

    // --- Mots-clés -------------------------------------------------------------
    Fn,
    Return,
    If,
    Else,
    ElseIf,
    While,
    Foreach,
    As,
    Break,
    Continue,
    Echo,
    True,
    False,
    /// Le mot-clé `Some`. Nommé `SomeKw` et pas `Some` : sinon un `use TokenKind::*`
    /// masquerait le `Some` d'`Option`, avec des erreurs très déroutantes.
    SomeKw,
    /// Le mot-clé `None` (même raison que `SomeKw`).
    NoneKw,

    // --- Délimiteurs ---------------------------------------------------------
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// `,`
    Comma,
    /// `:`
    Colon,

    // --- Opérateurs ----------------------------------------------------------
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `%`
    Percent,
    /// `.` (concaténation, comme en PHP)
    Dot,
    /// `=`
    Assign,
    /// `+=`
    PlusAssign,
    /// `-=`
    MinusAssign,
    /// `.=`
    DotAssign,
    /// `==`
    EqEq,
    /// `!=`
    NotEq,
    /// `<`
    Lt,
    /// `<=`
    LtEq,
    /// `>`
    Gt,
    /// `>=`
    GtEq,
    /// `&&`
    AndAnd,
    /// `||`
    OrOr,
    /// `!`
    Bang,
    /// `??`
    QuestionQuestion,
    /// `->`
    Arrow,
    /// `?->`
    QuestionArrow,
    /// `=>`
    FatArrow,
    /// `::`
    ColonColon,

    // --- Structure -------------------------------------------------------------
    /// Fin d'instruction (remplace le `;` de PHP).
    Newline,
    /// Fin du fichier.
    Eof,
}

impl TokenKind {
    /// Vrai si un retour à la ligne **juste après** ce jeton ne doit pas terminer
    /// l'instruction, parce que la ligne est visiblement incomplète :
    ///
    /// ```text
    /// $total = $prix +      <- la ligne finit par `+` : l'instruction continue
    ///          $port
    /// ```
    ///
    /// Renvoie `true` pour :
    /// - tous les opérateurs (`+ - * / % . = += -= .= == != < <= > >= && || ! ?? -> ?-> => ::`) ;
    /// - la virgule et les délimiteurs ouvrants `(`, `[`, `{` ;
    /// - `Newline` (deux lignes vides de suite ne produisent qu'un seul `Newline`).
    ///
    /// `false` pour tout le reste.
    pub fn is_continuation(&self) -> bool {
        todo!("is_continuation({self:?})")
    }
}

/// Si `word` est un mot-clé de webRu, renvoie le jeton correspondant, sinon `None`.
///
/// Les 15 mots-clés (sensibles à la casse) :
/// `fn return if else elseif while foreach as break continue echo true false Some None`
///
/// ```text
/// keyword("while")  -> Some(TokenKind::While)
/// keyword("Some")   -> Some(TokenKind::SomeKw)
/// keyword("count")  -> None
/// keyword("WHILE")  -> None
/// ```
pub fn keyword(word: &str) -> Option<TokenKind> {
    match word {
        "fn" => Some(TokenKind::Fn),
        "return" => Some(TokenKind::Return),
        "if" => Some(TokenKind::If),
        "else" => Some(TokenKind::Else),
        "elseif" => Some(TokenKind::ElseIf),
        "while" => Some(TokenKind::While),
        "foreach" => Some(TokenKind::Foreach),
        "as" => Some(TokenKind::As),
        "break" => Some(TokenKind::Break),
        "continue" => Some(TokenKind::Continue),
        "echo" => Some(TokenKind::Echo),
        "true" => Some(TokenKind::True),
        "false" => Some(TokenKind::False),
        "Some" => Some(TokenKind::SomeKw),
        "None" => Some(TokenKind::NoneKw),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Alias court pour alléger les tests : `K::Plus` au lieu de `TokenKind::Plus`.
    use TokenKind as K;

    #[test]
    fn mots_cles() {
        assert_eq!(keyword("fn"), Some(K::Fn));
        assert_eq!(keyword("return"), Some(K::Return));
        assert_eq!(keyword("if"), Some(K::If));
        assert_eq!(keyword("else"), Some(K::Else));
        assert_eq!(keyword("elseif"), Some(K::ElseIf));
        assert_eq!(keyword("while"), Some(K::While));
        assert_eq!(keyword("foreach"), Some(K::Foreach));
        assert_eq!(keyword("as"), Some(K::As));
        assert_eq!(keyword("break"), Some(K::Break));
        assert_eq!(keyword("continue"), Some(K::Continue));
        assert_eq!(keyword("echo"), Some(K::Echo));
        assert_eq!(keyword("true"), Some(K::True));
        assert_eq!(keyword("false"), Some(K::False));
        assert_eq!(keyword("Some"), Some(K::SomeKw));
        assert_eq!(keyword("None"), Some(K::NoneKw));
    }

    #[test]
    fn pas_des_mots_cles() {
        assert_eq!(keyword("count"), None);
        assert_eq!(keyword("WHILE"), None); // sensible à la casse
        assert_eq!(keyword("none"), None);
        assert_eq!(keyword(""), None);
        assert_eq!(keyword("iff"), None);
    }

    #[test]
    fn continuation() {
        for kind in [
            K::Plus,
            K::Minus,
            K::Dot,
            K::Assign,
            K::DotAssign,
            K::EqEq,
            K::AndAnd,
            K::QuestionQuestion,
            K::QuestionArrow,
            K::FatArrow,
            K::Comma,
            K::LParen,
            K::LBracket,
            K::LBrace,
            K::Newline,
        ] {
            assert!(
                kind.is_continuation(),
                "{kind:?} devrait être une continuation"
            );
        }
        for kind in [
            K::Int(1),
            K::Variable("x".to_string()),
            K::Ident("count".to_string()),
            K::RParen,
            K::RBracket,
            K::RBrace,
            K::True,
            K::Return,
            K::Eof,
        ] {
            assert!(
                !kind.is_continuation(),
                "{kind:?} ne devrait pas être une continuation"
            );
        }
    }
}
