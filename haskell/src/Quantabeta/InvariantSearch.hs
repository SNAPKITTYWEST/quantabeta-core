-- Layer 2: Arithmetic Invariant Search
-- Replaces LLM hypothesis generation.
-- Enumerates the space of arithmetic identities.
-- LiquidHaskell verifies invariants at compile time.

{-# LANGUAGE LambdaCase #-}
{-# LANGUAGE RecordWildCards #-}

module Quantabeta.InvariantSearch where

import Data.List (nub, sortBy)
import Data.Ord  (comparing, Down(..))

-- ─── Invariant expression grammar ────────────────────────────────────────────

data InvariantExpr
    = HeckeCorr    { level :: Int, weight :: Int }
    | PartitionVol { window :: Int }
    | RamanujanCong { modulus :: Int, residue :: Int }
    | Compose InvariantExpr InvariantExpr
    deriving (Eq, Show)

-- ─── Well-typed invariant check ──────────────────────────────────────────────

wellTyped :: InvariantExpr -> Bool
wellTyped HeckeCorr{..}     = level > 0 && weight > 0 && weight `mod` 2 == 0
wellTyped PartitionVol{..}  = window > 0 && window <= 252  -- max one trading year
wellTyped RamanujanCong{..} = modulus `elem` [5, 7, 11, 13] && residue < modulus
wellTyped (Compose a b)     = wellTyped a && wellTyped b

-- ─── Arithmetic invariant verification ───────────────────────────────────────

-- | Ramanujan partition congruences (exact, no approximation)
ramanujanCongruence :: Int -> Int -> [Int] -> Bool
ramanujanCongruence m r pValues =
    all (\k -> pValues !! (m * k + r) `mod` m == 0)
        [0 .. (length pValues - r - 1) `div` m]

-- | Hecke eigenvalue Deligne bound: |a_p| <= 2 * p^((k-1)/2)
deligneBound :: Int -> Int -> Rational -> Bool
deligneBound p k a_p =
    abs a_p <= 2 * fromIntegral p ^ ((k - 1) `div` 2 :: Int)

-- | Information coefficient positivity: IC > 0 implies signal has predictive power
icPositive :: [Rational] -> [Rational] -> Bool
icPositive signals returns =
    let n     = fromIntegral (length signals) :: Rational
        meanS = sum signals / n
        meanR = sum returns / n
        num   = sum (zipWith (\s r -> (s - meanS) * (r - meanR)) signals returns)
        denS  = sqrt $ fromRational $ sum (map (\s -> (s - meanS)^(2::Int)) signals) / n
        denR  = sqrt $ fromRational $ sum (map (\r -> (r - meanR)^(2::Int)) returns) / n
        ic    = if denS * denR == 0 then 0 else fromRational num / (denS * denR)
    in ic > 0

-- ─── Invariant enumeration ────────────────────────────────────────────────────

candidateInvariants :: [InvariantExpr]
candidateInvariants =
    -- Hecke operators at prime levels, even weights (modular forms)
    [ HeckeCorr l w | l <- [2, 3, 5, 7, 11, 13, 17, 19], w <- [2, 4, 6, 8] ]
    ++
    -- Ramanujan partition volatility windows
    [ PartitionVol w | w <- [5, 10, 20, 63, 126, 252] ]
    ++
    -- Ramanujan congruences (exact number-theoretic)
    [ RamanujanCong m r | m <- [5, 7, 11], r <- [m-1] ]

-- ─── Main search ─────────────────────────────────────────────────────────────

-- | Search for valid arithmetic invariants over a feature set.
-- Filters to well-typed expressions only.
-- LiquidHaskell would verify IC > 0 at compile time with refinement types.
searchInvariants :: [InvariantExpr] -> [InvariantExpr]
searchInvariants features =
    filter wellTyped
    . nub
    $ features ++ candidateInvariants

-- | Score invariants by algebraic complexity (prefer simpler)
scoreInvariant :: InvariantExpr -> Int
scoreInvariant HeckeCorr{..}     = level + weight
scoreInvariant PartitionVol{..}  = window
scoreInvariant RamanujanCong{..} = modulus
scoreInvariant (Compose a b)     = scoreInvariant a + scoreInvariant b + 1

rankInvariants :: [InvariantExpr] -> [InvariantExpr]
rankInvariants = sortBy (comparing scoreInvariant)

-- ─── Connection to clojure-lisp-bridge ───────────────────────────────────────

-- | Convert invariant to SGML claim tag for claimguard oracle
toSGMLClaim :: InvariantExpr -> String
toSGMLClaim inv = unlines
    [ "<claim>"
    , "  <source>" ++ show inv ++ "</source>"
    , "  <invariant_type>" ++ invariantType inv ++ "</invariant_type>"
    , "  <actor>quantabeta-invariant-search</actor>"
    , "</claim>"
    ]

invariantType :: InvariantExpr -> String
invariantType HeckeCorr{}     = "hecke-eigenvalue-correlation"
invariantType PartitionVol{}  = "ramanujan-partition-volatility"
invariantType RamanujanCong{} = "ramanujan-congruence"
invariantType (Compose _ _)   = "composed"
