% Entropy orchestration — Bifrost bridge.
% Calls Rust FFI for exact MPFR computation.
% Logs every computation to WORM chain.

:- module(entropy_bridge, [entropy/3, entropy_symbolic/2]).

% entropy(+Counts:list(int), -EntropyStr:string, -Interval:list(string))
% Calls Rust true-entropy crate via Bifrost FFI at 256-bit precision.
entropy(Counts, EntropyStr, [LoStr, HiStr]) :-
    % Validate inputs
    maplist(integer, Counts),
    maplist([C]>>(C >= 0), Counts),
    sum_list(Counts, Total),
    Total > 0,
    % Compute via Rust FFI (256-bit MPFR)
    bifrost_call(rust_entropy, compute_exact(Counts, 256), EntropyStr),
    bifrost_call(rust_entropy, compute_interval(Counts, 256), [LoStr, HiStr]),
    % WORM audit
    get_time(Timestamp),
    ledge_audit(entropy_computed, _{
        counts:    Counts,
        total:     Total,
        entropy:   EntropyStr,
        interval:  [LoStr, HiStr],
        precision: 256,
        timestamp: Timestamp
    }).

% entropy_symbolic(+Counts, -Terms)
% Returns symbolic representation without evaluation.
entropy_symbolic(Counts, Terms) :-
    sum_list(Counts, N),
    N > 0,
    include([C]>>(C > 0), Counts, PosCounts),
    maplist([C, term(C, N)]>>true, PosCounts, Terms).

% Sovereignty check: entropy must be below threshold for coherent system.
% Mirrors the Omega field threshold of 0.21.
entropy_coherent(Counts, MaxEntropy) :-
    entropy(Counts, EntropyStr, _),
    term_to_atom(Entropy, EntropyStr),
    Entropy < MaxEntropy.

% Partition entropy: entropy of complexity invariant distribution.
partition_entropy(N, Entropy, Interval) :-
    partition_numbers(N, PNums),
    entropy(PNums, Entropy, Interval).

% Euler pentagonal partition numbers.
partition_numbers(N, Ps) :-
    length(Ps, Len), Len is N + 1,
    partition_fill(0, N, [], Ps).

partition_fill(I, N, Acc, Result) :-
    I > N,
    reverse(Acc, Result).
partition_fill(I, N, Acc, Result) :-
    I =< N,
    partition_term(I, Acc, PI),
    I1 is I + 1,
    partition_fill(I1, N, [PI|Acc], Result).

partition_term(0, _, 1) :- !.
partition_term(I, Ps, PI) :-
    aggregate_all(sum(V),
        ( between(1, I, K),
          PentaPos is K * (3*K - 1) // 2,
          PentaPos =< I,
          Sign is ((-1)^(K+1)),
          Idx is I - PentaPos,
          nth0(Idx, Ps, Pk),
          V is Sign * Pk
        ),
        SumPos),
    aggregate_all(sum(V),
        ( between(1, I, K),
          PentaNeg is K * (3*K + 1) // 2,
          PentaNeg =< I,
          Sign is ((-1)^(K+1)),
          Idx is I - PentaNeg,
          nth0(Idx, Ps, Pk),
          V is Sign * Pk
        ),
        SumNeg),
    PI is SumPos + SumNeg.
