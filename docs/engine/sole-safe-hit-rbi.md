# E-03: sole unaided safe-hit RBI cause

Preregistered after verified local sole-HR checkpoint
`f600c52d8a634de043c4c174d8401565612a729e`, before tests/production changes.
[PROGRESS](../PROGRESS.md) owns status; the [approved matrix](g1-accounting-scope.md)
continues to require broader scoring causes, fielding/ER and complete goldens.

## Actual flow and primary rule

The canonical hit-value validator binds one supplied 1B/2B/3B classification to
Hit and a safe batter destination, including conservative ordinary walk-off
value ceilings. It does not prove why runners score. The preview supplies a
separate deterministic advancement policy, including truncated ordinary
walk-offs; it already awards RBI for these runs. The actor-local RBI fold still
correctly leaves a positive ordinary hit unknown without causal evidence.

2019 Rule9.04(a)(1), printed111/PDF123 in the retained text, awards RBI for runs
scoring unaided by an error as part of the play begun by a safe hit, unless
9.04(b) applies. Rule9.04(c) requires scorer judgment for a held ball/wrong-base
throw and distinguishes continuous advancement from stopping/restarting after
a misplay. This slice explicitly excludes those ambiguous misplay episodes,
retirements and force/reverse-force DP exceptions. Rulebook SHA256 remains
`d85e0e43cffaff4ed916065d5434b26b0d93d54dbd91630d585eeefdfca29e92`.

## Representation and deliberately bounded semantics

Append `Event.sole_safe_hit: Option<SoleSafeHitFacts>` after sole-HR facts in
BBE24, preserving existing tags. `SoleSafeHitFacts { batter_id }` asserts that
all represented advances/runs arise exclusively from this actual batter's
classified 1B/2B/3B, unaided by an error. Scoring is continuous advancement in
the play begun by the safe hit, not a separate earlier/later episode, an
error-assisted advance or a stopped/restarted misplay decision under9.04(c).
This is explicit input evidence, not a deduction from net destinations/RBI.

Presence requires ordinary PlateAppearance, actual-batter PA/Hit and exactly
one explicit 1B/2B/3B modifier. All before occupants and the batter have exactly
one movement, preserving identities/origins/carried charges. Before runners
hold or advance safely; batter ends safely at First/Second/Third (not Home).
Zero outs, no personnel changes/third-out or other occurrence/retirement facts,
and no SB/CS/WP/PB/Balk/PO/A units belong to this proof. Existing hit-value,
collision, legal-run, participation and review constraints remain in force;
this does not broaden unknown-review extra-base hits or walk-off permissions.

Independently derive named RBI0–3 from validated before-runner movements to
Home, not delta.runs or supplied RBI units. Require exactly those actual-batter
RBI units and canonical run conservation. Expose `sole_safe_hit_rbi()` and add
it to the generic positive fallback, leaving the accounting fold unchanged.
Missing facts retain positive RBI None despite Hit/value/NoMisplay/correct
supplied units. Generic independently known zero-run RBI stays unchanged.

Known actor/classification/role/origin/charge/destination/unit contradictions
precede unrelated chronology/review uncertainty. Reuse existing family guards;
unbound richer records retain their existing legal/partial/unsupported
semantics, not universal illegality. Unknown review with a legal single or an
unrelated earlier foul-chance error can preserve this independent unaided RBI
cause without certifying E/PO/A/ER. No earned-run reconstruction change.

## Observable acceptance checks fixed before implementation

1. Three classified values × eight masks × three outs × both halves × separate
   movement/credit order permutations: literal named RBI, physical movements,
   current/old pitcher charges and actor-local fold on constructor/reducer/codec.
   Independently compute expectations; keep before immutable.
2. Held runners, unequal safe advances (single from First to Home), legal
   Unknown-review singles, prior foul-chance error, ordinary walk-offs and
   truncated winning-run advancement. Existing extra-base Unknown and ambiguous
   walk-off ceilings remain unsupported, not silently expanded.
3. Missing cause/correct or absent supplied RBI stay partial; wrong actors,
   missing/duplicate/foreign Hit/value/RBI, origins/charges, outs, retreats,
   unrepresented occupants, Home-ending batter and competing causes reject on
   all canonical entry paths with independently framed valid wire controls.
   Encoding/version/tag errors stay distinct from semantic failures.
4. Actor-local unknown irreversibility and runner-only independence persist;
   new RBI facts do not manufacture ER or full official-box completeness.
5. Before producer changes, add safe-hit-only binding assertions to its existing
   1,296-context property and witness semantic red. Bind the existing policy,
   including its ordinary walk-off truncation, without changing outcomes,
   movement/credits, probabilities/RNG, versions or accepted seed1 presentation.
6. Trace every None/header/tail migration preserving historical fixtures and
   assertions. Run all applicable Rust/fmt/strict all-target Clippy/Python/native/
   offline/audit/diagnostic/CLI/prose/diff/frozen checks; inspect exact staged
   ownership/equality before another local checkpoint.

No independent review, push, new data, G1/full-box or probability/UI claim.
