% Layer 3: Factor Synthesis — Prolog DCG
% Input: Verified InvariantExpr (from Haskell FFI)
% Output: Rust struct + backtest harness + Bifrost audit manifest

:- module(factor_synthesis, [synthesize_factor/2, invariant_to_ast/2]).

% ─── Invariant → AST ─────────────────────────────────────────────────────────

invariant_to_ast(hecke_corr(Level, Weight), ast(hecke, Level, Weight)).
invariant_to_ast(partition_vol(Window), ast(partition, Window, 0)).
invariant_to_ast(ramanujan_congruence(Mod, Residue), ast(congruence, Mod, Residue)).

% ─── AST → Rust Code (DCG) ───────────────────────────────────────────────────

:- use_module(library(dcg/basics)).

rust_feature(ast(hecke, Level, _Weight)) -->
    "pub fn factor_hecke_", integer(Level),
    "(series: &[Rational]) -> Rational {\n",
    "    hecke_cross_correlation(series, series, ", integer(Level), ")\n",
    "}\n".

rust_feature(ast(partition, Window, _)) -->
    "pub fn factor_partition_vol_", integer(Window),
    "(returns: &[Rational]) -> Vec<Rational> {\n",
    "    compute_partition_volatility(returns, ", integer(Window), ")\n",
    "}\n".

rust_feature(ast(congruence, Mod, Residue)) -->
    "// Ramanujan congruence: p(", integer(Mod), "k+", integer(Residue),
    ") ≡ 0 mod ", integer(Mod), "\n",
    "pub fn factor_ramanujan_congruence(n: u64) -> bool {\n",
    "    let p = partition_numbers(n as usize);\n",
    "    let target = (n % ", integer(Mod), " == ", integer(Residue), ") as u64;\n",
    "    (p[n as usize].clone() % Integer::from(", integer(Mod), ")).is_zero() || target == 0\n",
    "}\n".

% ─── Main synthesis predicate ────────────────────────────────────────────────

synthesize_factor(Invariant, FactorCode) :-
    invariant_to_ast(Invariant, AST),
    phrase(rust_feature(AST), CodeChars),
    atom_chars(RustCode, CodeChars),
    term_to_atom(AST, ASTAtom),
    term_hash(ASTAtom, Hash),
    FactorCode = factor{
        ast:       AST,
        rust_code: RustCode,
        hash:      Hash,
        invariant: Invariant
    }.

% ─── Bifrost manifest generation ─────────────────────────────────────────────

bifrost_manifest(Factor, Manifest) :-
    Factor = factor{ast: _, rust_code: Code, hash: Hash, invariant: Inv},
    atom_length(Code, Len),
    format(atom(Manifest),
        '{"factor_id":"QB-~w","invariant":"~w","code_hash":"~w","code_len":~w}',
        [Hash, Inv, Hash, Len]).

% ─── Tests ───────────────────────────────────────────────────────────────────

:- begin_tests(factor_synthesis).

test(hecke_synthesis) :-
    synthesize_factor(hecke_corr(11, 2), Factor),
    Factor = factor{ast: ast(hecke, 11, 2), rust_code: _, hash: _, invariant: _}.

test(partition_synthesis) :-
    synthesize_factor(partition_vol(20), Factor),
    Factor = factor{ast: ast(partition, 20, 0), rust_code: _, hash: _, invariant: _}.

:- end_tests(factor_synthesis).
