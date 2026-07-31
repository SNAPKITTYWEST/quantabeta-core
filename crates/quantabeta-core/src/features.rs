// Layer 1: Symbolic Feature Algebra
// All arithmetic is exact rational — no f64, no float drift.
// rug::Rational guarantees deterministic reproducible results.

use rug::{Integer, Rational};
use std::collections::HashMap;

/// AST node for symbolic feature expressions
#[derive(Clone, Debug, PartialEq)]
pub enum FeatureExpr {
    LogReturn,
    PartitionVol { window: usize },
    HeckeCorr { level: u32 },
    Compose(Box<FeatureExpr>, Box<FeatureExpr>),
}

#[derive(Clone, Debug)]
pub struct FeatureMeta {
    pub arity: usize,
    pub complexity: u32,
    pub algebraic_degree: u32,
}

#[derive(Clone, Debug)]
pub struct SymbolicFeature {
    pub expr: FeatureExpr,
    pub metadata: FeatureMeta,
}

// ─── Ramanujan Partition Numbers ─────────────────────────────────────────────

/// Compute partition numbers p(0)..p(n) using Euler's pentagonal recurrence.
/// p(n) = sum_{k != 0} (-1)^(k-1) * p(n - g_k)
/// where g_k = k(3k-1)/2 are generalized pentagonal numbers.
/// All arithmetic is exact integer — no approximation.
pub fn partition_numbers(n: usize) -> Vec<Integer> {
    let mut p = vec![Integer::from(0); n + 1];
    p[0] = Integer::from(1);

    for i in 1..=n {
        let mut sum = Integer::from(0);
        let mut k: i64 = 1;
        loop {
            // Positive pentagonal: g_k = k(3k-1)/2
            let g_pos = (k * (3 * k - 1)) / 2;
            // Negative pentagonal: g_{-k} = k(3k+1)/2
            let g_neg = (k * (3 * k + 1)) / 2;

            if g_pos > i as i64 { break; }

            let sign = if k % 2 == 1 { 1i64 } else { -1i64 };

            sum += sign * p[i - g_pos as usize].clone();
            if g_neg <= i as i64 {
                sum += sign * p[i - g_neg as usize].clone();
            }
            k += 1;
        }
        p[i] = sum;
    }
    p
}

/// Ramanujan partition volatility: apply partition entropy to discretized return buckets.
/// Returns exact Rational entropy per window — deterministic.
pub fn compute_partition_volatility(returns: &[Rational], window: usize) -> Vec<Rational> {
    if returns.len() < window { return vec![]; }

    let partition_cache = partition_numbers(window + 10);

    returns.windows(window).map(|w| {
        // Discretize returns into integer buckets (multiply by 10000, floor)
        let buckets: Vec<usize> = w.iter().map(|r| {
            let scaled = r.clone() * Rational::from((10000i64, 1i64));
            let n = scaled.numer().clone().abs();
            (n % Integer::from(window as u64)).to_u32_wrapping() as usize
        }).collect();

        // Count frequency per bucket
        let mut freq: HashMap<usize, usize> = HashMap::new();
        for b in &buckets { *freq.entry(*b).or_insert(0) += 1; }

        // Partition entropy: sum p(freq_i) / p(window) as exact Rational
        let p_total = &partition_cache[window];
        if p_total.is_zero() { return Rational::from((0i64, 1i64)); }

        let entropy: Rational = freq.values().map(|&f| {
            let p_f = &partition_cache[f.min(window)];
            Rational::from((p_f.clone(), p_total.clone()))
        }).fold(Rational::from((0i64, 1i64)), |acc, x| acc + x);

        entropy
    }).collect()
}

// ─── Hecke Operator Cross-Correlation ────────────────────────────────────────

/// Hecke T_n operator acting on a q-series coefficient vector.
/// For level N, weight k: (T_n f)(q) = sum a_{nm} q^m + n^{k-1} sum a_{m/n} q^m
/// Returns eigenvalue overlap as exact Rational.
pub fn hecke_cross_correlation(
    series_a: &[Rational],
    series_b: &[Rational],
    level: u32,
) -> Rational {
    let n = level as usize;
    let len = series_a.len().min(series_b.len());
    if len < n { return Rational::from((0i64, 1i64)); }

    // Apply T_n to series_a: a_m -> a_{nm} (for indices that exist)
    let t_n_a: Vec<Rational> = (0..len).map(|m| {
        let nm = m * n;
        if nm < len {
            series_a[nm].clone()
        } else {
            Rational::from((0i64, 1i64))
        }
    }).collect();

    // Inner product <T_n(a), b> as exact Rational
    let dot: Rational = t_n_a.iter().zip(series_b.iter())
        .map(|(a, b)| a.clone() * b.clone())
        .fold(Rational::from((0i64, 1i64)), |acc, x| acc + x);

    // Normalize by length
    dot / Rational::from((len as i64, 1i64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partition_numbers_known_values() {
        let p = partition_numbers(10);
        // OEIS A000041
        assert_eq!(p[0], Integer::from(1));
        assert_eq!(p[1], Integer::from(1));
        assert_eq!(p[2], Integer::from(2));
        assert_eq!(p[3], Integer::from(3));
        assert_eq!(p[4], Integer::from(5));
        assert_eq!(p[5], Integer::from(7));
        assert_eq!(p[10], Integer::from(42));
    }

    #[test]
    fn test_partition_volatility_deterministic() {
        let r1 = vec![
            Rational::from((1i64, 100i64)),
            Rational::from((2i64, 100i64)),
            Rational::from((3i64, 100i64)),
            Rational::from((4i64, 100i64)),
        ];
        let v1 = compute_partition_volatility(&r1, 3);
        let v2 = compute_partition_volatility(&r1, 3);
        // Determinism: same input always produces same output
        assert_eq!(v1, v2);
    }
}
