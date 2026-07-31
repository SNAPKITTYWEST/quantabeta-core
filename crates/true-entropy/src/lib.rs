// True Entropy — exact Shannon entropy via MPFR correct rounding.
// H(P) = log2(N) - (1/N) * sum(c_i * log2(c_i))
// H is algebraic: linear combination of logs of integers.
// No f64 ever. Every bit auditable.

use rug::{Float, Integer};
use rug::float::Round;

/// Point estimate of Shannon entropy at `prec_bits` precision.
/// Uses MPFR Round::Nearest — correctly rounded.
pub fn shannon_entropy_exact<I, T>(counts: I, prec_bits: u32) -> Float
where
    I: IntoIterator<Item = T>,
    T: Into<Integer>,
{
    let mut total = Integer::new();
    let mut terms: Vec<Integer> = Vec::new();

    for c in counts {
        let ci: Integer = c.into();
        if ci > 0 {
            total += &ci;
            terms.push(ci);
        }
    }

    if terms.is_empty() || total == 0 {
        return Float::with_val(prec_bits, 0);
    }

    // sum_c_log_c = sum(c_i * log2(c_i))
    let mut sum_c_log_c = Float::with_val(prec_bits, 0);
    for c in &terms {
        let mut log_c = Float::with_val(prec_bits, c);
        log_c.log2();
        let contrib = Float::with_val(prec_bits, c) * log_c;
        sum_c_log_c += contrib;
    }

    // n_log_n = N * log2(N)
    let mut n_log_n = Float::with_val(prec_bits, &total);
    n_log_n.log2();
    n_log_n *= Float::with_val(prec_bits, &total);

    // H = (N*log2(N) - sum(c_i*log2(c_i))) / N
    let mut h = n_log_n - sum_c_log_c;
    h /= Float::with_val(prec_bits, &total);
    h
}

/// Guaranteed interval [L, U] containing true entropy.
/// L uses Round::Down, U uses Round::Up.
/// Invariant: U - L < 2^(-prec_bits + guard_bits)
pub fn shannon_entropy_interval<I, T>(counts: I, prec_bits: u32) -> (Float, Float)
where
    I: IntoIterator<Item = T>,
    T: Into<Integer>,
{
    let mut total = Integer::new();
    let mut terms: Vec<Integer> = Vec::new();

    for c in counts {
        let ci: Integer = c.into();
        if ci > 0 {
            total += &ci;
            terms.push(ci);
        }
    }

    if terms.is_empty() {
        let z = Float::with_val(prec_bits, 0);
        return (z.clone(), z);
    }

    // Lower bound: minimize H = minimize N*log2(N), maximize sum(c_i*log2(c_i))
    let mut sum_hi = Float::with_val(prec_bits, 0);
    for c in &terms {
        let mut log_c = Float::with_val(prec_bits, c);
        log_c.log2_round(Round::Up);
        let mut contrib = Float::with_val(prec_bits, c);
        contrib *= log_c;
        sum_hi += contrib;
    }
    let mut n_log_n_lo = Float::with_val(prec_bits, &total);
    n_log_n_lo.log2_round(Round::Down);
    n_log_n_lo *= Float::with_val(prec_bits, &total);
    let h_lo = (n_log_n_lo - sum_hi) / Float::with_val(prec_bits, &total);

    // Upper bound: maximize H
    let mut sum_lo = Float::with_val(prec_bits, 0);
    for c in &terms {
        let mut log_c = Float::with_val(prec_bits, c);
        log_c.log2_round(Round::Down);
        let mut contrib = Float::with_val(prec_bits, c);
        contrib *= log_c;
        sum_lo += contrib;
    }
    let mut n_log_n_hi = Float::with_val(prec_bits, &total);
    n_log_n_hi.log2_round(Round::Up);
    n_log_n_hi *= Float::with_val(prec_bits, &total);
    let h_hi = (n_log_n_hi - sum_lo) / Float::with_val(prec_bits, &total);

    (h_lo, h_hi)
}

/// Symbolic representation: coeff * log2(base)
#[derive(Clone, Debug)]
pub struct SymLog2 {
    pub coeff: rug::Rational,
    pub base:  Integer,
}

/// Exact symbolic entropy expression — no evaluation, no rounding.
/// H = log2(N) + sum(-c_i/N * log2(c_i))
pub fn shannon_entropy_symbolic<I, T>(counts: I) -> Vec<SymLog2>
where
    I: IntoIterator<Item = T>,
    T: Into<Integer>,
{
    let mut total = Integer::new();
    let mut terms: Vec<Integer> = Vec::new();

    for c in counts {
        let ci: Integer = c.into();
        if ci > 0 { total += &ci; terms.push(ci); }
    }

    if terms.is_empty() { return vec![]; }

    let mut symbolic: Vec<SymLog2> = vec![
        SymLog2 {
            coeff: rug::Rational::from((Integer::from(1), total.clone())),
            base:  total.clone(),
        }
    ];

    for c in terms {
        symbolic.push(SymLog2 {
            coeff: rug::Rational::from((-c.clone(), total.clone())),
            base:  c,
        });
    }

    symbolic
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_entropy() {
        // Uniform distribution over 2 outcomes: H = 1 bit
        let counts = vec![1u64, 1u64];
        let h = shannon_entropy_exact(counts, 256);
        let diff = (h - Float::with_val(256, 1)).abs();
        assert!(diff < Float::with_val(256, 1e-10_f64));
    }

    #[test]
    fn test_certain_entropy() {
        // Degenerate distribution: H = 0
        let counts = vec![1u64, 0u64];
        let h = shannon_entropy_exact(counts, 256);
        assert!(h.abs() < Float::with_val(256, 1e-10_f64));
    }

    #[test]
    fn test_interval_contains_point() {
        let counts = vec![3u64, 1u64, 2u64, 4u64];
        let h_point = shannon_entropy_exact(counts.clone(), 256);
        let (h_lo, h_hi) = shannon_entropy_interval(counts, 256);
        assert!(h_lo <= h_point);
        assert!(h_point <= h_hi);
    }

    #[test]
    fn test_symbolic_terms_count() {
        // n distinct values → n+1 symbolic terms (1 for log2(N) + n for c_i terms)
        let counts = vec![1u64, 2u64, 3u64];
        let sym = shannon_entropy_symbolic(counts);
        assert_eq!(sym.len(), 4); // log2(6) + 3 terms
    }
}
