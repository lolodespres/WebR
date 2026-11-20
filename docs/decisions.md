# Décisions de conception de webRu

Les choix de langage déjà tranchés, avec leur raison, pour ne pas les rediscuter sans le savoir.

## D1 — Pas de variables dynamiques (2026-11-20)

`$$nom` et `${$expr}` n'existent pas. À la place : un tableau associatif.

```php
$valeurs[$nom] = "x"
echo $valeurs[$nom] ?? "absent"
```

Raisons :
- le parser doit pouvoir vérifier les types et les `Option` de chaque variable ;
- sécurité : un visiteur ne doit jamais pouvoir choisir quelle variable est lue ou écrasée ;
- PHP a dû changer le sens de `$$a['x']` entre PHP 5 et PHP 7, une rupture contraire à
  « Don't break the web » ;
- chaque variable peut être associée à une case de l'arène avant l'exécution.

## D2 — Pas d'appel de fonction par son nom (2026-11-20)

`$nom()` avec une chaîne de caractères, ou `$$nom()`, n'existent pas. Les fonctions sont des
**valeurs** (syntaxe de PHP 8.1) :

```php
$salut = bonjour(...)         // la fonction elle-même, de type fn(string): string
echo $salut("Léa")
```

Raisons : signatures vérifiables ; un visiteur ne peut pas faire appeler une fonction
arbitraire. Le routage dynamique passe par un tableau de fonctions avec repli :
`$pages[$page] ?? page404(...)`.

## D3 — Variantes de comportement : interfaces avec méthodes par défaut (2026-11-20)

Pas de mécanisme qui chercherait `faitUnTrucAlaManiereDe{$spec}` et se replierait sur
`faitUnTrucBase`. Quand les classes arriveront :
- une interface fournit les implémentations par défaut ;
- une classe par variante redéfinit seulement ce qui change ;
- la variante est choisie **une seule fois**, à un seul endroit.

Raisons : une faute de frappe ou une signature différente devient une erreur détectée au lieu
d'un repli silencieux sur la version de base ; le code reste trouvable par une recherche. C'est le
même motif que le trait `SessionStore` du moteur.

## D4 — Gestion native des erreurs (2026-11-20)

Trois niveaux, du plus général au plus précis.

**1. Par défaut : message global en fin de requête.** Une erreur non traitée remonte jusqu'à la
requête, qui s'arrête proprement et affiche le message global (configurable une fois). Les autres
requêtes ne sont pas touchées.
- En développement : tous les détails (fichier, ligne, cause).
- En production : un message neutre pour le visiteur, les détails dans les logs.

**2. Par fonction : un `catch` attaché au corps, à la manière de Ruby.** Il remplace le
comportement par défaut pour cette fonction. Il doit renvoyer une valeur du type de la fonction,
ou laisser remonter l'erreur (vérifié par le parser).

```php
fn afficherProfil(int $id): string {
    $user = chargerUtilisateur($id)
    return "Bonjour {$user->nom}"
} catch $e {
    return "Profil indisponible pour le moment"
}
```

**3. Par ligne : `or fail "message"`.** Le message est écrit sur l'étape qui peut échouer (comme
le `.context(...)` de la crate Rust `anyhow`).

```php
fn afficherCommande(int $id): string {
    $user = chargerUtilisateur($id) or fail "Utilisateur introuvable"
    $commande = chargerCommande($user) or fail "Commande indisponible"
    return "Commande n° {$commande->numero}"
}
```

Raison commune : une erreur ne doit jamais être transformée en valeur inventée (`0.0`, chaîne
vide…). Ce serait le problème du `null` silencieux sous une autre forme.

Propositions à valider :
- le message de `fail` est **destiné au visiteur** (écrit volontairement par le développeur, donc
  affichable même en production) ; la cause technique d'origine est conservée, mais seulement
  dans les logs ;
- `or fail` fonctionne aussi sur une `Option` : `None` devient une erreur. On aurait ainsi `??`
  pour donner une valeur par défaut et `or fail` pour signaler une absence anormale. Exemple :
  `$user = Database::find($id) or fail "Utilisateur introuvable"`.

Encore ouvert : comment une fonction annonce qu'elle peut échouer (exceptions implicites comme en
PHP, ou type visible `Result<T, Erreur>` dans la signature). À trancher avec l'interpréteur.
