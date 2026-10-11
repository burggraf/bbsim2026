# E-03 scoring facts and field coverage

Implementation requirements for the [approved E-03/G1 matrix](g1-accounting-scope.md). This note maps fields to facts and missing-fact behavior; it neither changes the approved vocabulary/exclusions nor certifies the current implementation. [PROGRESS.md](../PROGRESS.md) is the sole live status ledger.

## Coverage boundary

There are three different claims:

1. **Physical continuity:** validated transitions join and the game starts/ends legally. Existing `GameAccounting::complete_game` proves only initial-to-terminal coverage.
2. **Credited segment accounting:** fold supplied named credits and explicit classifications. Existing `credited_*` counters describe those supplied credits; zero does not prove that an official statistic was fully observed. Existing AB/ER options preserve additional uncertainty.
3. **Complete agreed boxes:** every required field is independently derivable from complete, consistent event/scorer facts for the approved bounded vocabulary. This proof is not implemented yet. It must include physical continuity, scoring coverage and supported-context checks, not simply rename claim 1 or 2.

A future complete-box result must identify missing field/fact coverage or unsupported context when certification fails. Partial logs remain usable through the existing qualified segment view. An omitted scoring fact is not interchangeable with an explicit reviewed absence. Unknown source facts cannot be upgraded by a default, an empty credit vector, a source summary total, or a bare completeness assertion. Authoritative scorer judgments are permitted inputs, but they must be tied to the actual event/actor and checked against all represented contextual facts.

## Per-field derivation and mandatory facts

“Complete coverage” below means all relevant opportunities in the observed game are accounted for, including reviewed absences and earlier chances within an event. It does not mean every physically possible outcome occurs. This table specifies evidence requirements, not new unchecked boolean flags or a mandatory wire layout.

| Field | Independent derivation | Mandatory facts for a complete cell; behavior when missing |
| --- | --- | --- |
| Winner / terminal state | Legal final state and final named-run totals | Initial state, continuous events, supported legal ending and non-tied final score. Partial/unsupported ending cannot certify a game winner. |
| Inning / final R | Named legally scored runners, grouped by before-state offense/half | Physical crossings, explicit final-out/cause/order facts when relevant, observed half boundaries and checked ending. Unplayed/unobserved half stays distinct from observed zero; initial snapshot runs are not segment runs. |
| Team H / E | Sum complete player H / E cells | Complete hit classifications and all relevant fielding chances/credits. Existing sums of supplied credits are not complete team totals. |
| Batting PA / pitching BF | Actual PA participants, once per PA; runner-only/personnel events add none | All PA and non-PA events, actual batter/current pitcher and any represented responsibility exceptions. Missing events or excluded mid-count exceptions prevent complete cells. |
| AB / AB against | Count PA except BB, HBP, SH, SF and interference/obstruction awards | Mutually consistent explicit PA/scorer classification; generic physical out/reach alone is insufficient. Current unclassified-PA segment AB stays `None`. |
| Batting R / pitching R | Named Run credits and each scorer's carried pitcher charge | Legal run reconciliation plus complete reach/relief/FC/pinch-runner responsibility history. Actual pitcher participation is not substituted for a carried charge. Unknown lineage cannot certify charged R. |
| H / H allowed | One Hit for each scorer-classified hit PA | Actual batter/current pitcher, complete hit-versus-no-hit classification, force/out/scorer constraints. An omitted Hit credit is not a reviewed no-hit outcome. |
| 1B / 2B / 3B / HR, batting and pitching | Partition complete H by explicit scored hit value | Rule 9.06 value and applicable throw/overslide/appeal/walk-off facts; out-of-park HR already supplies an explicit subtype. Final destination or number of runners advancing alone cannot classify a generic Hit. Any unclassified credited Hit leaves its segment breakdown unknown, not zero. |
| TB | `1B + 2*2B + 3*3B + 4*HR` | Known classified-hit components and H-component conservation. Any unknown component makes TB unknown; do not sum physical runner advances. |
| RBI | Named, cause-qualified RBI units | Legal runs plus explicit RBI cause/scorer judgment, double-play/error and other applicable exclusions under Rule 9.04. A run or supplied total alone does not establish RBI; absence is not reviewed zero. |
| BB / BB allowed | Count classified walks | Complete PA classifications, eligibility and named actual participants. IBB is a subset, not a second walk added to BB. Missing walk intentionality does not erase an otherwise known BB, but prevents complete IBB. |
| IBB / IBB allowed | Subset of BB explicitly classified as intentional | Deliberate/ordinary walk distinction. Missing distinction is unknown, not ordinary by default. No manager or random-IBB policy is implemented by this statistic. |
| K, batting and pitching | Classified strikeouts, including eligible uncaught-third-strike outcomes | Complete PA classification, before-state first occupancy/outs, relevant catch/touch/out facts. Do not infer K from Plate→Out or infer safe-reach eligibility from first vacating during the play. |
| HBP / HBP allowed | Explicit HBP PA classification | Named participants and legal award; complete PA coverage. A first-base reach alone cannot distinguish HBP from BB, interference, hit or other cause. |
| SH / SF, batting and pitching | Explicit scorer-classified sacrifices | Required contact/catch/advance/out/cause facts and applicable scorer judgment, not simply a runner advance or RBI. Unclassified contact cannot certify sacrifice zeroes. |
| SB | Named qualifying steal occurrences | Attempt/cause facts distinguish steals from hit/force/FC/WP/PB/balk/error advances and support simultaneous effects. Movement to a higher base alone is not a complete steal classification. |
| CS | Named qualifying caught-stealing occurrences | Attempt/occurrence facts, actual retirement or explicitly represented error-saved attempt under Rule 9.07(h). CS counts episodes, not universally physical outs; unknown summarized multiplicity remains unsupported. |
| GIDP | Explicit qualifying ground-double-play PA classification | Contact type, force/reverse-force cause, individual outs and sufficient sequence/scorer facts. Two physical outs alone are not GIDP; K+CS and other two-out controls remain distinct. |
| Pitching outs / IP | Actual event outs attributed to the before-state pitcher; IP formatted from integer outs | All outs, including runner-only events, and supported participation history. No decimal-innings addition; fielding PO conservation requires separate complete credit coverage. |
| Pitching ER | Rule 9.16 reconstruction per responsible pitcher | Observed half origin/end, all relevant errors/PB/missed chances, hypothetical outs/advances and relief/inherited/FC responsibility treatment. Missing review or unsupported counterfactual history keeps uncertain charged ER `None`. Published ER is comparison evidence, not reconstructed event evidence; pitcher ER need not sum to team-earned runs. |
| Pitching WP / balks | Named scorer-qualified occurrences charged to actual pitcher | Explicit occurrence and causal advancement facts, including simultaneous steal exceptions (9.07(a)) and WP/PB distinctions. A runner advance or generic pitch-event label alone is not complete counter coverage. The [accepted bounded WP/PB checkpoint](pitch-occurrences.md) supplies single-delivery qualification, not complete coverage or all contexts. The accepted [ordinary exact-award balk checkpoint](balk-occurrences.md) binds one imposed602a occurrence, not all causes/progress/appeals or complete counter coverage. The [accepted all-safe nonterminal play-proceeds checkpoint](balk-progress.md) supplies named actual minimum touches, not a complete occurrence/box certificate; independent pitch/error/SB binding still needs facts. |
| Fielding PO | Named actual putout units | Every actual retirement, eligible before-defense actor and required chance/role facts. Same fielder can receive multiple legitimate PO units; complete PO totals must reconcile physical outs. |
| Fielding A | Named qualifying assist units | Throw/deflection chains, actual or error-prevented putout, applicable scorer exclusions and known eligible actors. Do not infer assist identities or multiplicity from PO or vector order. |
| Fielding E | Named qualifying error units across all relevant chances | Misplay/chance facts and scorer classification, including earlier muffed fouls and error-avoided outs. An absent Error credit is not reviewed zero. |
| Fielding PB | Named scorer-qualified passed-ball occurrences | Actual catcher identity or checked role precondition, pitch/advance cause and steal exceptions; before-defense attribution. Known DH/pitcher exclusions alone do not prove another fielder was the catcher. |

## Integration and acceptance checks

- Preserve existing partial/source fixtures and their explicit unknown review/responsibility facts. Do not silently add missing historical credits/classifications.
- Validate new facts in the shared canonical constructor used by reducer and semantic decode. The fold derives from immutable validated events, not trusted final boxes or after-state stat totals.
- Reject an explicit contradiction as illegal; report a legal but insufficiently represented context as unsupported. Missing optional facts may remain usable in the partial view but cannot certify affected complete fields.
- Verify per-field conservation only when its inputs are complete: H components/TB, PA/AB components, named R/line/score/charge totals and outs/PO. Do not impose a universal CS≤outs or blanket credit deduplication.
- Complete-game golden tests must supply literal expected agreed player/team/inning cells, including justified zeroes and an unplayed bottom ninth. Omission/forgery controls must demonstrate that physical completeness and `NoMisplay` alone cannot certify those boxes.
- This requirements map is not implementation or test evidence. Remaining hit-type contexts, missing counters, complete-box coverage reporting, misplay/occurrence reconstruction and final matrix evidence remain work within E-03. Credited hit-value, IBB and [bounded two-out GIDP](two-out-contact.md) slices are accepted checkpoints, not complete-box proof.
