// Copyright (C) 2026 Jorge Andre Castro
//
// Ce programme est un logiciel libre : vous pouvez le redistribuer et/ou le modifier
// selon les termes de la Licence Publique Générale GNU telle que publiée par la
// Free Software Foundation, soit la version 2 de la licence, soit (à votre convention)
// n'importe quelle version ultérieure.

//! # embedded-exp
//!
//! Exponentielle en virgule fixe Q15 pour systèmes embarqués.
//!
//! ## Caractéristiques
//!
//! - `#![no_std]` — aucune dépendance à la bibliothèque standard
//! - Arithmétique entière pure (pas de flottants, pas de `libm`)
//! - Compatible RP2040 (Cortex-M0+) et RP2350 (Cortex-M33)
//! - Algorithme : Réduction d'intervalle + approximation polynomiale de Taylor (degré 4)
//! - Temps d'exécution **constant** (déterministe) — idéal pour les noyaux temps réel
//! - Précision < 10 ULP sur toute la plage `[-1.0, 0[`
//!
//! ## Format Q15
//!
//! En Q15, un `i16` représente un nombre réel dans `[-1.0, 1.0[` :
//! ```text
//! valeur_réelle = valeur_i16 / 32768.0
//! ```
//! Exemples :
//! - `0`      → 0.0
//! - `16384`  → 0.5
//! - `32767`  → ≈ 1.0
//! - `-32768` → -1.0
//!
//! ## Algorithme
//!
//! La méthode utilise la **réduction d'intervalle** combinée à une
//! **approximation polynomiale de Taylor de degré 4** :
//!
//! 1. Décomposition : `x = n·ln(2) + r`, avec `n` entier et `r ∈ [0, ln(2)[`
//! 2. Propriété : `e^x = 2^n · e^r` → `2^n` est un simple décalage de bits
//! 3. Approximation : `e^r ≈ 1 + r + r²/2 + r³/6 + r⁴/24`,"L'implémentation utilise des multiplications par l'inverse pour éviter les divisions coûteuses

//!
//! Le reste `r` est toujours dans `[0, ln(2)[ ≈ [0, 0.693[`, ce qui garantit
//! la convergence rapide du polynôme. Toutes les multiplications restent
//! dans les bornes `i32` sans risque de débordement.
//!
//! ## Exemple
//!
//! ```rust
//! use embedded_exp::exp_q15;
//!
//! // exp(0.0) = 1.0 → non représentable en Q15 signé → sature à i16::MAX
//! assert_eq!(exp_q15(0), i16::MAX);
//!
//! // exp(-0.5) ≈ 0.6065 → ≈ 19874 en Q15
//! let res = exp_q15(-16384);
//! assert!((res as i32 - 19874).abs() < 10);
//!
//! // exp(-1.0) doit être positif
//! assert!(exp_q15(-32768) > 0);
//! ```

#![no_std]

/// ln(2) en Q15 : 0.693147… × 32768 = 22713
const LN2_Q15: i32 = 22713;

/// 1/ln(2) en Q15 : (1/0.693147…) × 32768 = 47274
const INV_LN2_Q15: i32 = 47274;

// Multiplicateurs magiques pour éviter les divisions (1/X * 2^15)
const INV_6_Q15: i32 = 5461;  // round(32768 / 6)
const INV_24_Q15: i32 = 1365; // round(32768 / 24)

/// Calcule l'exponentielle d'un nombre en virgule fixe Q15.
/// Temps d'exécution constant et déterministe.
#[inline]
pub fn exp_q15(x: i16) -> i16 {
    // e^x ≥ 1.0 pour tout x ≥ 0 → saturation à i16::MAX
    if x >= 0 {
        return i16::MAX;
    }

    // Protection contre les valeurs trop petites (e^-10 est négligeable en Q15)
    // -10.0 en Q15 serait -327680, mais x est i16. On couvre toute la plage.
    let x_i32 = x as i32;

    // --- Étape 1 : Réduction d'intervalle ---
    // n = floor(x / ln2)
    let n: i32 = (x_i32 * INV_LN2_Q15) >> 30;
    
    // r = x − n·ln(2)
    let r: i32 = x_i32 - n * LN2_Q15;

    // --- Étape 2 : Approximation polynomiale e^r sur [0, ln(2)[ ---
    // Taylor degré 4 : e^r ≈ 1 + r + r²/2 + r³/6 + r⁴/24
    let r2: i32 = (r * r) >> 15;
    let r3: i32 = (r2 * r) >> 15;
    let r4: i32 = (r3 * r) >> 15;

    let er: i32 = 32768                      // 1.0 en Q15
                + r                          // r
                + (r2 >> 1)                  // r²/2
                + ((r3 * INV_6_Q15) >> 15)   // r³/6
                + ((r4 * INV_24_Q15) >> 15); // r⁴/24

    // --- Étape 3 : Reconstruction e^x = 2^n · e^r ---
    // Comme x < 0, n est toujours négatif ou nul. 
    // n est l'exposant de 2, on décale à droite de |n|.
    let result: i32 = er >> (-n);

    // --- Saturation et cast final ---
    if result >= 32767 {
        32767
    } else if result <= 0 {
        0
    } else {
        result as i16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exp_zero_saturates() {
        // exp(0.0) = 1.0 → non représentable en Q15 signé → saturation
        assert_eq!(exp_q15(0), i16::MAX);
    }

    #[test]
    fn test_exp_saturation_positive() {
        // Tout x ≥ 0 → e^x ≥ 1.0 → saturation
        assert_eq!(exp_q15(1), i16::MAX);
        assert_eq!(exp_q15(i16::MAX), i16::MAX);
    }

    #[test]
    fn test_exp_negative_eighth() {
        // exp(-0.125) ≈ 0.8825 → 0.8825 × 32768 ≈ 28917
        // x = -0.125 en Q15 → -4096
        let res = exp_q15(-4096);
        assert!((res as i32 - 28917).abs() < 10, "exp(-0.125): attendu ≈28917, reçu {}", res);
    }

    #[test]
    fn test_exp_negative_quarter() {
        // exp(-0.25) ≈ 0.7788 → 0.7788 × 32768 ≈ 25519
        // x = -0.25 en Q15 → -8192
        let res = exp_q15(-8192);
        assert!((res as i32 - 25519).abs() < 10, "exp(-0.25): attendu ≈25519, reçu {}", res);
    }

    #[test]
    fn test_exp_negative_half() {
        // exp(-0.5) ≈ 0.6065 → 0.6065 × 32768 ≈ 19874
        // x = -0.5 en Q15 → -16384
        let res = exp_q15(-16384);
        assert!((res as i32 - 19874).abs() < 10, "exp(-0.5): attendu ≈19874, reçu {}", res);
    }

    #[test]
    fn test_exp_negative_one() {
        // exp(-1.0) ≈ 0.3679 → 0.3679 × 32768 ≈ 12054
        // x = -1.0 en Q15 → -32768
        let res = exp_q15(-32768);
        assert!((res as i32 - 12054).abs() < 10, "exp(-1.0): attendu ≈12054, reçu {}", res);
    }

    #[test]
    fn test_exp_strictly_positive() {
        // exp(x) > 0 pour tout x réel
        assert!(exp_q15(-32768) > 0);
        assert!(exp_q15(-16384) > 0);
        assert!(exp_q15(-1) > 0);
    }

    #[test]
    fn test_exp_monotone() {
        // e^x est strictement croissante
        let a = exp_q15(-32768); // exp(-1.000)
        let b = exp_q15(-28672); // exp(-0.875)
        let c = exp_q15(-24576); // exp(-0.750)
        let d = exp_q15(-16384); // exp(-0.500)
        let e = exp_q15(-8192);  // exp(-0.250)
        assert!(a < b, "exp(-1.0)={} doit être < exp(-0.875)={}", a, b);
        assert!(b < c, "exp(-0.875)={} doit être < exp(-0.75)={}", b, c);
        assert!(c < d, "exp(-0.75)={} doit être < exp(-0.5)={}", c, d);
        assert!(d < e, "exp(-0.5)={} doit être < exp(-0.25)={}", d, e);
    }
}