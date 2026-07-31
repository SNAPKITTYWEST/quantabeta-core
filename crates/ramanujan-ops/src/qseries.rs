// q-Series and Rogers-Ramanujan identities.
// q-Oscillator algebra: aa† - q*a†a = q^(-N)
// At q = root of unity → topological quantum invariants.
// All arithmetic exact Rational.

use rug::{Integer, Rational};

/// q-integer [n]_q = (q^n - q^(-n)) / (q - q^(-1))
/// Represented as exact Rational when q is rational.
pub fn q_integer(n: u32, q: &Rational) -> Rational {
    if *q == Rational::from((1i64, 1i64)) {
        return Rational::from((n as i64, 1i64));
    }

    let q_n   = q_power(q, n as i64);
    let q_neg = q_power(q, -(n as i64));
    let q_pos = q.clone();
    let q_m1  = q_power(q, -1);

    let num = q_n - q_neg;
    let den = q_pos - q_m1;

    if den == Rational::from((0i64, 1i64)) {
        Rational::from((n as i64, 1i64))
    } else {
        num / den
    }
}

/// q^n as exact Rational (for rational q)
pub fn q_power(q: &Rational, n: i64) -> Rational {
    if n == 0 { return Rational::from((1i64, 1i64)); }
    if n > 0 {
        let mut r = Rational::from((1i64, 1i64));
        for _ in 0..n { r *= q.clone(); }
        r
    } else {
        let mut r = Rational::from((1i64, 1i64));
        for _ in 0..(-n) { r /= q.clone(); }
        r
    }
}

/// q-Pochhammer symbol (a; q)_n = prod_{k=0}^{n-1} (1 - a*q^k)
/// Exact Rational for rational a, q.
pub fn q_pochhammer(a: &Rational, q: &Rational, n: u32) -> Rational {
    let mut result = Rational::from((1i64, 1i64));
    for k in 0..n {
        let q_k = q_power(q, k as i64);
        let term = Rational::from((1i64, 1i64)) - a.clone() * q_k;
        result *= term;
    }
    result
}

/// Rogers-Ramanujan first identity:
/// sum_{n>=0} q^(n^2) / (q;q)_n = prod_{n>=0} 1/((1-q^(5n+1))(1-q^(5n+4)))
/// Verify up to order `prec` with rational q = p/r
pub fn rogers_ramanujan_check(q: &Rational, prec: u32) -> bool {
    // LHS: sum q^(n^2) / (q;q)_n up to n where n^2 < prec
    let mut lhs = Rational::from((0i64, 1i64));
    let mut n = 0u32;
    while n * n < prec {
        let q_n2 = q_power(q, (n * n) as i64);
        let poch = q_pochhammer(q, q, n);
        if poch != Rational::from((0i64, 1i64)) {
            lhs += q_n2 / poch;
        }
        n += 1;
    }

    // RHS: truncated product
    let mut rhs = Rational::from((1i64, 1i64));
    let mut k = 0u32;
    while 5 * k + 4 < prec {
        let q1 = Rational::from((1i64, 1i64)) - q_power(q, (5 * k + 1) as i64);
        let q4 = Rational::from((1i64, 1i64)) - q_power(q, (5 * k + 4) as i64);
        if q1 != Rational::from((0i64, 1i64)) && q4 != Rational::from((0i64, 1i64)) {
            rhs /= q1 * q4;
        }
        k += 1;
    }

    // Check LHS ≈ RHS (exact rational comparison up to truncation)
    let diff = (lhs - rhs).abs();
    diff < Rational::from((1i64, 1000000i64)) // tolerance for truncation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_q_integer_at_1() {
        let q = Rational::from((1i64, 1i64));
        assert_eq!(q_integer(5, &q), Rational::from((5i64, 1i64)));
    }

    #[test]
    fn test_q_pochhammer_n0() {
        let q = Rational::from((1i64, 2i64));
        let a = Rational::from((1i64, 1i64));
        assert_eq!(q_pochhammer(&a, &q, 0), Rational::from((1i64, 1i64)));
    }

    #[test]
    fn test_rogers_ramanujan_small_q() {
        // q = 1/10 is small enough for rapid convergence
        let q = Rational::from((1i64, 10i64));
        assert!(rogers_ramanujan_check(&q, 20));
    }
}
