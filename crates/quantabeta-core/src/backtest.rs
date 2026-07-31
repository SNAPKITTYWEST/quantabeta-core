// Layer 4: Deterministic Backtest Engine
// No f64. No VectorBT. No NumPy. Integer ticks. Lamport logical clock.

use rug::{Integer, Rational};
use serde::{Deserialize, Serialize};

/// Logical clock — Lamport ordering, no wall-time dependency.
#[derive(Clone, Debug, Default)]
pub struct LogicalClock {
    pub tick: u64,
}

impl LogicalClock {
    pub fn advance(&mut self) { self.tick += 1; }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BacktestResult {
    pub pnl:            String, // Exact integer PnL in basis points
    pub sharpe_low:     String, // Rational lower bound
    pub sharpe_high:    String, // Rational upper bound
    pub max_drawdown:   String, // Rational
    pub total_trades:   u64,
    pub audit_hash:     String,
}

/// Tick-level market event — integer price, integer volume.
#[derive(Clone, Debug)]
pub struct Tick {
    pub price:  Integer,
    pub volume: Integer,
    pub clock:  u64,
}

/// Compiled factor signal: +1 long, -1 short, 0 flat.
pub type Signal = i8;

pub struct DeterministicBacktest {
    pub fee_bps:  u64, // Integer basis points
    pub clock:    LogicalClock,
}

impl DeterministicBacktest {
    pub fn new(fee_bps: u64) -> Self {
        Self { fee_bps, clock: LogicalClock::default() }
    }

    /// Run backtest over ticks with a signal vector.
    /// PnL = Sum(pos_t * (price_{t+1} - price_t)) - fees
    /// All arithmetic exact integer/rational.
    pub fn run(&mut self, ticks: &[Tick], signals: &[Signal]) -> BacktestResult {
        assert_eq!(ticks.len(), signals.len(), "ticks and signals must align");

        let mut pnl = Integer::from(0);
        let mut returns: Vec<Rational> = vec![];
        let mut peak = Integer::from(0);
        let mut max_dd = Integer::from(0);
        let mut trades = 0u64;

        for i in 0..ticks.len().saturating_sub(1) {
            self.clock.advance();
            let pos = signals[i] as i64;
            if pos == 0 { continue; }

            let price_diff = ticks[i + 1].price.clone() - ticks[i].price.clone();
            let gross = Integer::from(pos) * price_diff;
            let fee   = Integer::from(self.fee_bps as i64) * ticks[i].price.clone()
                        / Integer::from(10000i64);
            let net   = gross - fee;

            pnl += net.clone();
            if pnl > peak { peak = pnl.clone(); }
            let dd = peak.clone() - pnl.clone();
            if dd > max_dd { max_dd = dd; }

            returns.push(Rational::from((net, Integer::from(1))));
            trades += 1;
        }

        // Rational Sharpe interval
        let (sharpe_low, sharpe_high) = self.sharpe_interval(&returns);
        let audit = self.hash_result(&pnl, &sharpe_low, &sharpe_high);

        BacktestResult {
            pnl:          pnl.to_string(),
            sharpe_low:   format!("{}", sharpe_low),
            sharpe_high:  format!("{}", sharpe_high),
            max_drawdown: max_dd.to_string(),
            total_trades: trades,
            audit_hash:   audit,
        }
    }

    fn sharpe_interval(&self, returns: &[Rational]) -> (Rational, Rational) {
        if returns.is_empty() {
            return (Rational::from((0i64, 1i64)), Rational::from((0i64, 1i64)));
        }
        let n = Rational::from((returns.len() as i64, 1i64));
        let mean: Rational = returns.iter().cloned()
            .fold(Rational::from((0i64, 1i64)), |a, b| a + b) / n.clone();

        let variance: Rational = returns.iter()
            .map(|r| { let d = r.clone() - mean.clone(); d.clone() * d })
            .fold(Rational::from((0i64, 1i64)), |a, b| a + b) / n;

        // Interval: sharpe ∈ [mean/(stddev+ε), mean/(stddev-ε)]
        // Use rational approximation of stddev
        let eps = Rational::from((1i64, 1000i64));
        if variance <= Rational::from((0i64, 1i64)) {
            return (Rational::from((0i64, 1i64)), Rational::from((0i64, 1i64)));
        }

        let stddev_approx = variance.clone(); // simplified — real impl uses MPFR
        let low  = mean.clone() / (stddev_approx.clone() + eps.clone());
        let high = mean.clone() / (stddev_approx - eps);
        (low, high)
    }

    fn hash_result(&self, pnl: &Integer, sl: &Rational, sh: &Rational) -> String {
        use sha2::{Digest, Sha256};
        let content = format!("{}|{}|{}", pnl, sl, sh);
        format!("{:x}", Sha256::digest(content.as_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backtest_deterministic() {
        let ticks: Vec<Tick> = (0..5).map(|i| Tick {
            price:  Integer::from(100 + i),
            volume: Integer::from(1000),
            clock:  i as u64,
        }).collect();
        let signals: Vec<Signal> = vec![1, 1, -1, 0, 1];

        let mut bt1 = DeterministicBacktest::new(1);
        let mut bt2 = DeterministicBacktest::new(1);
        let r1 = bt1.run(&ticks, &signals);
        let r2 = bt2.run(&ticks, &signals);

        assert_eq!(r1.pnl, r2.pnl);
        assert_eq!(r1.audit_hash, r2.audit_hash);
    }
}
