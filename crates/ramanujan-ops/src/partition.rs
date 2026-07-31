// Hardy-Ramanujan-Rademacher partition function p(n).
// Exact integer computation via Euler pentagonal recurrence.
// Invariant: p(5k+4) ≡ 0 (mod 5) — Ramanujan congruence, checkable.

use rug::Integer;

/// Compute p(0)..p(n) exactly via pentagonal recurrence.
/// Time: O(n * sqrt(n)). Space: O(n).
/// Verified against OEIS A000041.
pub fn partition_numbers(n: usize) -> Vec<Integer> {
    let mut p = vec![Integer::from(0); n + 1];
    p[0] = Integer::from(1);

    for i in 1..=n {
        let mut sum = Integer::from(0);
        let mut k: i64 = 1;
        loop {
            let g_pos = (k * (3 * k - 1)) / 2;
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

/// Ramanujan congruence check: p(5k+4) ≡ 0 (mod 5) for all k.
/// Returns true if all values in range satisfy the congruence.
pub fn verify_ramanujan_congruence_5(max_k: usize) -> bool {
    let max_n = 5 * max_k + 4;
    let p = partition_numbers(max_n);
    (0..=max_k).all(|k| {
        let n = 5 * k + 4;
        (p[n].clone() % Integer::from(5i64)) == 0
    })
}

/// Ramanujan congruence: p(7k+5) ≡ 0 (mod 7)
pub fn verify_ramanujan_congruence_7(max_k: usize) -> bool {
    let max_n = 7 * max_k + 5;
    let p = partition_numbers(max_n);
    (0..=max_k).all(|k| {
        let n = 7 * k + 5;
        (p[n].clone() % Integer::from(7i64)) == 0
    })
}

/// Complexity invariant: exact state count for combinatorial search space of size n.
/// Returns p(n) as exact Integer — deterministic complexity bound.
pub fn complexity_invariant(n: usize) -> Integer {
    partition_numbers(n)[n].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oeis_a000041() {
        let p = partition_numbers(20);
        let expected = [1,1,2,3,5,7,11,15,22,30,42,56,77,101,135,176,231,297,385,490,627];
        for (i, &e) in expected.iter().enumerate() {
            assert_eq!(p[i], Integer::from(e as i64), "p({}) mismatch", i);
        }
    }

    #[test]
    fn test_ramanujan_congruence_5() {
        assert!(verify_ramanujan_congruence_5(10));
    }

    #[test]
    fn test_ramanujan_congruence_7() {
        assert!(verify_ramanujan_congruence_7(5));
    }
}
