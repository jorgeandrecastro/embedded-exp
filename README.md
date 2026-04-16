# embedded-exp

[![Crates.io](https://img.shields.io/crates/v/embedded-exp.svg)](https://crates.io/crates/embedded-exp)
[![License: GPL-2.0](https://img.shields.io/badge/License-GPL%20v2-blue.svg)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html)

**Exponentielle en virgule fixe Q15 ultra-rapide et déterministe pour systèmes embarqués.**

---

## Update Version 0.1.2 
Choix de #![forbid(unsafe_code)] pour plus de fiabilité et choix de opt-level = 3 dans Cargo.toml pour une vitesse maximale.

## ⚠️ Disclaimer : Domaine de définition

Cette crate est optimisée pour le format **Q15 signé** (plage `[-1.0, 1.0[`). 
En raison des contraintes mathématiques de ce format :
- **Entrées négatives (x < 0)** : Précision maximale (< 10 ULP). Idéal pour les fonctions d'activation (Sigmoïde, Softmax) et la décroissance de signaux.
- **Entrées positives (x \geq 0)** : Puisque e^x dépasse immédiatement 1.0, la fonction **sature systématiquement à `i16::MAX`** (0.9999...).

## 🚀 Pourquoi embedded-exp ?

Dans le monde de l'embarqué (`no_std`), le calcul de l'exponentielle est souvent soit trop lourd (flottants/libm), soit imprécis. `embedded-exp` propose une approche **purement entière** optimisée pour les processeurs sans unité de calcul flottant (FPU) comme le RP2040 (Cortex-M0+), tout en brillant sur le RP2350 (Cortex-M33).

### Points forts :
* **Performance critique** : Zéro division. Utilise des multiplications par l'inverse pour un gain de cycles massif.
* **Déterminisme** : Temps d'exécution constant, indispensable pour les noyaux temps réel (RTOS).
* **Empreinte minimale** : Aucune dépendance, pas de table de recherche (LUT) géante, juste des mathématiques pures.
* **Précision** : Erreur maximale < 10 ULP sur toute la plage utile.

## 📊 Algorithme

La crate utilise une **réduction d'intervalle** couplée à une **approximation polynomiale de Taylor de degré 4** :
1. `x = n·ln(2) + r`
2. `e^x = 2^n · e^r` (le facteur 2^n est géré par un simple décalage de bits).
3. `e^r` est calculé via Taylor sur l'intervalle [0, \ln(2)[.

## 🛠 Utilisation

Ajoutez ceci à votre `Cargo.toml` :

```toml
[dependencies]
embedded-exp = "0.1.2"
Exemple simple :
```rust
use embedded_exp::exp_q15;

fn main() {
    // exp(-0.5) en Q15
    // -0.5 réel = -16384 en Q15 (i16)
    let result = exp_q15(-16384);
    
    // Résultat : ~19874 (soit ~0.6065 réel)
    println!("Résultat Q15 : {}", result);
}
```
⚖️ Licence
Copyright (C) 2026 Jorge Andre Castro.

Ce programme est un logiciel libre : vous pouvez le redistribuer et/ou le modifier selon les termes de la Licence Publique Générale GNU telle que publiée par la Free Software Foundation, soit la version 2 de la licence, soit (à votre convention) n'importe quelle version ultérieure.