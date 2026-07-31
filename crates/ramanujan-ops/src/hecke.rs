// Hecke operators T_n acting on modular form q-expansions.
// Double coset formula: (T_n f)(q) = sum_{ad=n, 0<=b<d} d^(k-1) * f|_k [a b; 0 d]
// All Fourier coefficients exact Integer. No float.
// Sovereign utility: Galois-certifiable spectra for verification.

use rug::Integer;

/// Modular form as q-expansion: coefficients a_0, a_1, ..., a_N
/// f(q) = sum_{n=0}^{N} a_n * q^n
#[derive(Clone, Debug)]
pub struct ModularForm {
    pub level:  u32,
    pub weight: u32,
    pub coeffs: Vec<Integer>, // a_n, exact
}

impl ModularForm {
    pub fn new(level: u32, weight: u32, coeffs: Vec<Integer>) -> Self {
        Self { level, weight, coeffs }
    }

    pub fn prec(&self) -> usize { self.coeffs.len() }
}

/// Hecke operator T_n
#[derive(Clone, Debug)]
pub struct HeckeOperator {
    pub index:  u64,  // n in T_n
    pub weight: u32,  // k
}

impl HeckeOperator {
    pub fn new(n: u64, weight: u32) -> Self {
        Self { index: n, weight }
    }

    /// Apply T_n to f, returning coefficients up to prec.
    /// T_n(f) has a_m = sum_{d | gcd(n,m)} d^(k-1) * a_{nm/d^2}
    pub fn apply(&self, f: &ModularForm, prec: usize) -> ModularForm {
        let n = self.index;
        let k = self.weight;
        let mut result = vec![Integer::from(0); prec];

        for m in 0..prec {
            // sum over d dividing gcd(n, m)
            let g = gcd(n, m as u64);
            let mut coeff = Integer::from(0);
            for d in divisors(g) {
                let idx = (n as usize) * m / (d * d) as usize;
                if idx < f.prec() {
                    let d_pow = Integer::from(d).pow(k - 1);
                    coeff += d_pow * f.coeffs[idx].clone();
                }
            }
            result[m] = coeff;
        }

        ModularForm::new(f.level, f.weight, result)
    }
}

/// Deligne bound check: |a_p| <= 2 * p^((k-1)/2)
/// Returns true if the bound holds for the given prime p and coefficient.
pub fn deligne_bound_holds(a_p: &Integer, p: u64, k: u32) -> bool {
    let exp = (k - 1) / 2;
    let bound = Integer::from(2i64) * Integer::from(p).pow(exp);
    a_p.clone().abs() <= bound
}

// ─── helpers ──────────────────────────────────────────────────────────────────

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn divisors(n: u64) -> Vec<u64> {
    let mut divs = vec![];
    let mut i = 1u64;
    while i * i <= n {
        if n % i == 0 {
            divs.push(i);
            if i != n / i { divs.push(n / i); }
        }
        i += 1;
    }
    divs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hecke_t2_identity() {
        // T_1 is the identity operator
        let coeffs: Vec<Integer> = (0..10).map(Integer::from).collect();
        let f = ModularForm::new(1, 12, coeffs.clone());
        let t1 = HeckeOperator::new(1, 12);
        let result = t1.apply(&f, 10);
        for i in 0..10 {
            assert_eq!(result.coeffs[i], coeffs[i]);
        }
    }

    #[test]
    fn test_deligne_bound() {
        // For weight 12, prime 2: |a_2| <= 2 * 2^5 = 64
        let a2 = Integer::from(24i64); // Ramanujan tau(2) = -24... wait, it's 24 for Delta
        assert!(deligne_bound_holds(&a2, 2, 12));
    }
}
