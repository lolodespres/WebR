# Cahier des charges & Architecture — webRu

Nouveau langage Web inspiré de PHP, écrit en Rust. Nom de code : **webRu**.

## 1. Vision et objectifs du projet

Créer un langage de programmation moderne dédié au web, inspiré par l'ergonomie et la
simplicité de PHP, mais entièrement conçu et motorisé en Rust.

- **Zéro breaking change** (règle « Don't break the web ») : compatibilité ascendante et
  descendante garantie grâce à un système d'**éditions** (à la manière de Rust), permettant de
  faire évoluer le langage sans jamais casser le code existant.
- **Cycles de vie longs (LTS)** : éviter l'obsolescence rapide des versions grâce à une
  séparation stricte entre le cœur du moteur (VM/parseur) et les extensions (base de données,
  réseau, crypto).
- **Apprentissage de Rust par la pratique** : utiliser ce projet comme fil rouge pour monter en
  compétence sur l'écosystème Rust.

## 2. Caractéristiques clés du langage

### Syntaxe familière et épurée

- Conservation du `$` pour les variables (facilite la lecture, l'interpolation dans les chaînes
  de caractères et simplifie le travail du lexer).
- Suppression du point-virgule (`;`) : le retour à la ligne marque la fin d'une instruction
  (avec gestion intelligente des sauts de ligne après un opérateur ou dans des parenthèses).

### Suppression totale du `null`

- Élimination des erreurs fatales liées aux références nulles.
- Utilisation native du type `Option<T>` (`Some($valeur)` ou `None`), directement mappé sur
  l'`Option` de Rust.
- Support natif des opérateurs ergonomiques comme `??` (valeur par défaut) et `?->` (chaînage
  optionnel).

### Système de sessions utilisateurs natif

- Conservation de la simplicité d'utilisation d'une variable superglobale (`$session` ou
  `$_SESSION`).
- Architecture modulaire basée sur un trait `SessionStore` en Rust : stockage par défaut en
  mémoire/SQLite (zéro configuration), avec bascule transparente vers Redis ou des cookies
  chiffrés.

### Points forts conservés de PHP

- Typage fort et explicite.
- Tableaux associatifs puissants et natifs.
- Simplicité de déploiement et cycle de vie isolé par requête HTTP.

## 3. Exemple de syntaxe cible

```php
// Typage fort, pas de point-virgule, gestion native de l'absence de valeur
fn getUserName(int $id): Option<string> {
    $user = Database::find($id)

    // Retourne None automatiquement si $user est absent
    return $user?->name
}

// Gestion native et sécurisée de la session
$session['visits'] = ($session['visits'] ?? 0) + 1
$name = getUserName(42) ?? "Visiteur anonyme"

echo "Bienvenue $name, visite n° {$session['visits']}"
```

## 4. Architecture technique (moteur Rust)

### Gestion de la mémoire (arena allocation)

- Toutes les variables d'une requête sont allouées dans un vecteur unique (`Vec<Value>`) et
  manipulées via des index entiers (`usize`).
- Avantage : contourne la complexité du borrow checker de Rust sur les structures
  auto-référentielles et permet de vider toute la mémoire instantanément à la fin de la
  requête HTTP.

### Découpage du projet Cargo

- **lexer** : découpe le code source en jetons (tokens) en gérant le `$` et les retours à la
  ligne significatifs (crate recommandée : `logos`).
- **parser** : construit l'arbre syntaxique abstrait (AST) et vérifie la cohérence des types et
  des `Option`.
- **runtime** : exécute l'AST, gère l'arène mémoire, le driver de `$_SESSION` et le serveur
  HTTP intégré (crate recommandée : `axum`).

## 5. Feuille de route

1. **Initialisation** : choisir un nom de code pour le projet et créer le dépôt Git / Cargo.
2. **Version 0.1 (tree-walk interpreter)** : coder un premier interpréteur simple exécutant
   directement l'AST (variables, conditions, boucles, fonctions de base) sans passer par du
   bytecode complexe.
3. **Intégration web** : connecter l'interpréteur au serveur HTTP `axum` et implémenter le
   premier driver `$_SESSION`.
4. **Standard library & éditions** : ajouter les fonctions essentielles (chaînes, tableaux,
   JSON, base de données via `sqlx`) et figer la première édition du langage.
