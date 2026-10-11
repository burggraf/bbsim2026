# E-03 / G1 accounting acceptance matrix

**Approved completion contract, 2026-10-11.** The maintainer first chose D-10 option B, complete agreed box fields for a bounded event vocabulary, in interview `021d9e0d-b0b7-4164-a7c8-8bd88774c071`, then selected **“Approve the proposed matrix”** in confirmation interview `e75959e7-704f-40cc-943b-1a3a10e34285`. The fields, event families, exclusions and observable evidence below are approved without edits. [PROGRESS.md](../PROGRESS.md) remains the sole live status ledger. This scope approval does **not** complete E-03 or sign off G1.

## Meaning of complete

- A fully specified simulated game starts at the initial empty first/top state, records every relevant event and scoring fact, and ends legally. Its agreed box fields below must be known and independently derivable—not merely credited lower bounds or default zeroes.
- Physical before/after state, `FieldingReview::NoMisplay`, and `GameAccounting::complete_game` do **not** by themselves certify complete scoring facts. A nominally complete game cannot hide omitted classifications, fielding credits, missed chances or responsibility exceptions.
- Existing partial/source logs remain usable with explicit coverage and unknowns. Missing facts must not be fabricated to make a historical fixture complete. In particular, the bounded F-02 rows do not supply all fielder identities or historical review facts.
- Supplied event/scorer facts can include authoritative classifications and counterfactual judgments where necessary. Their attribution, consistency and required coverage must be validated; a trusted final box total or an unchecked “complete” label is not independent accounting.
- Unsupported **legal** histories remain unsupported, not structurally impossible. This G1 boundary does not waive D-09's positive-probability requirement; P-01 must preregister its taxonomy and identify prerequisites before sampling categories that need further representation.

## 1. Agreed fields

All counters are per player and team where applicable. Pitching outs are stored exactly; baseball IP notation is derived from outs, not decimal arithmetic.

| Line | Required fields | Existing foundation / missing proof or representation |
| --- | --- | --- |
| Game/line score | Winner/terminal state, runs in each observed half, final runs, team hits and errors; distinguish unplayed half from observed zero | Legal endings and line fold exist; require complete-field reconciliation, not just scoreboard agreement. |
| Batting | PA, AB, R, H, 1B, 2B, 3B, HR, RBI, BB, IBB, K, HBP, SH, SF, SB, CS, GIDP; total bases derived from classified hits | Most credited counters and conditional AB exist. Hit-value, IBB and GIDP classification/counters are missing; supplied generic Hit plus final base cannot establish hit value. Required scoring coverage/conditions are not certified. |
| Pitching | BF, AB against, outs/IP, H and hit-type breakdown, R, ER, BB, IBB, K, HBP, WP, balks, SH and SF allowed | BF/outs/charged R and many credited fields exist. Additional classifications/counters and complete-field proof are needed. ER is currently proved only for the bounded clean vocabulary or zero segment charges. |
| Fielding | PO, A, E, PB | Credited fold and known DH/pitcher-role restrictions exist. Require complete named credits and legal attribution/multiplicity for supported plays; do not infer catcher/position facts from missing data. |

**Not required at G1 under this contract:** pitcher W/L/S/HLD decisions, complete-game/shutout awards, seasonal standings/usage, all secondary official statistics, derived rate displays, defense range/arm estimation or every defensive-position transfer. These are explicit exclusions, not assertions that the statistics are impossible or unimportant. Pitcher ER is required; a separate team-earned-run field is not. Do not equate summed pitcher ER with team-earned runs.

## 2. Event vocabulary

This is an input/representation contract, not a random-outcome generator. No RNG, ratings, manager or UI work is part of E-03.

| Family | Included bounded shapes / required facts |
| --- | --- |
| Non-contact PA | Caught/uncaught K with eligibility and supplied touch/out facts; ordinary BB and deliberate IBB; HBP; explicit first-base interference/obstruction awards. Preserve PA/AB/BF and known responsibility. |
| Contact PA | Explicit scorer-classified 1B/2B/3B/out-of-park HR; ordinary BIP out, supported two-out ground/force/reverse-force double plays with cause/order facts; SH/SF; reach-on-error and supported error-assisted advancement. Do not infer hit value, GIDP, RBI or sacrifice classification merely from final destinations. |
| Fielder's choice | Existing lone occupied runner retired, batter first, one out/no runs, including successive replacements across events and carried pitcher charge. Multi-runner FC charge substitutions stay excluded until separately represented. |
| Runner/pitch events | Steal success, actual CS, **error-saved CS without a physical out**, pickoff/attempted advancement, WP, PB, balk and supported advances. Supply enough occurrence/cause facts for official credits and earned-run reconstruction. Earlier distinct episodes normally get their own non-PA transitions; simultaneous final-PA effects stay one PA. |
| Personnel | Existing bench/removed eligibility, retained batting slots, pinch-runner slot/charge inheritance, supported DH/no-DH pitcher changes and irreversible pitcher-for-DH loss. Substitutions affecting batter/pitcher credit are between PAs; unsupported mid-count responsibility exceptions are excluded. |
| Scoring/endings | Existing regulation/extras without a post-2019 automatic runner, third-out force/before-first/timing/after-first and bounded preceding-runner appeal facts, ordinary winning-run walk-offs and out-of-park HR exception. Include ordinary multi-out plays only when the final-out/cause/timing facts are sufficient. |

For included error/PB/missed-chance shapes, complete scoring facts must support the applicable Rule 9.16 reconstruction and relief/inherited-runner treatment. An event with a known Error but unknown earlier chances is not automatically fully reviewed. Unsupported counterfactual advancement/ordering must remain visible; it cannot enter the complete simulated-log subset with ER silently unknown or invented.

**Explicit G1 exclusions:** fourth-out/elective-appeal sequences; ambiguous multiple-out/force order; multi-runner FC responsibility exchanges; complex unrepresented interference/obstruction or counterfactual histories; mid-count batter/pitcher statistic exceptions; full position-transfer/double-switch/pitcher-runner combinations; suspended/tied/forfeit/shortened or otherwise unsupported replay endings. The diagnostic replay-candidate audit is not a completeness certificate. Exclusions must be machine-detectable or stated as checked input preconditions, not inferred from coalesced source zeroes.

## 3. Observable acceptance evidence

1. **Field/coverage contract:** each required field has a derivation, mandatory facts and missing-fact behavior. Tests show incomplete physical/scorer logs cannot masquerade as complete boxes. Preserve current partial-log behavior and codec semantic validation.
2. **Independent complete golden:** a fully specified synthetic game with literal expected player/team/inning boxes, including a normal 8½-inning home win. Every required zero is justified by known event coverage. The old 52-event PA/run/out proof alone does not satisfy this check.
3. **Conditional golden/property cases:** hit values/AB exclusions/IBB/GIDP, legal RBI causes, PO/A/E/PB attribution, actual and error-saved CS, pitching WP/balk, inherited/FC charges, clean and misplay-affected ER, third-out scoring and terminal halves. Include positive controls and wrong/omitted-fact rejections, all relevant masks/outs/halfs and order permutations.
4. **Conservation:** PA/AB components, classified H/TB components, exact pitching outs and complete PO totals, named runs/score/line totals and pitcher charges agree independently. CS is an occurrence count, not a universal physical-out bound. No blanket deduplication of valid credit units.
5. **Historical evidence with honest coverage:** reconcile permitted supported event/field cells against bounded source/golden references, reporting unavailable facts and shared-upstream lineage. The existing 54 box/27 ER comparisons are supplemental source evidence, not complete engine boxes or independent season-wide accuracy.
6. **Fresh verification and signoff:** all applicable Rust/Python/native/audit checks and targeted substantive review; record failures/remaining exclusions. E-03 completion and a separate maintainer G1 go/no-go require the approved matrix and resulting evidence. Thousands of randomized games belong to P-02, not an invented prerequisite for this deterministic G1 slice.

## Implementation sequence

Lock coverage and scoring-fact requirements → hit-value/IBB/GIDP and missing counters → supported credit/context constraints and complete-field golden → explicit misplay/occurrence facts and bounded ER reconstruction → final supported historical/property reconciliation and G1 evidence packet. Split further into small test-first E-03 slices in the live ledger; do not start probability fitting or UI work early.
