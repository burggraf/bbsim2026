# E-03 — reviewed exact BB/HBP forced-award earned-run proof

## Selected before TDD

This follows the [clean ground retirement proof](clean-ground-er.md) and the
[clean safe-PA contract](clean-safe-pa-er.md). Its separately excluded forced
multi-runner awards are now the next bounded proof, not a retroactive rewrite of
that earlier contract. Actual preview seed1 play35 demonstrates the gap: a
reviewed bases-loaded walk followed by HR leaves positive charged ER unknown.

2019 Rules 5.05(b)(1)–(2), 5.06(b)(3)(B) and 9.16(a) make the ordinary first-base
BB/HBP award and its required forced advances part of errorless play. Select
only a generic PA with supplied Walk or HBP, no outs/personnel/extra declared
contact, pitch or balk episode, and exact protected movements:

- Batter safely Plate→First with canonical current-pitcher responsibility.
- The uninterrupted occupied chain starting at First advances one base each;
  its Third occupant scores only when all three before bases were occupied.
- Every other before runner holds, either omitted from the movement vector or
  explicitly moved to the same base. No excess/unforced advance or retirement
  is certified. Movement vector order supplies no chronology.
- Existing canonical credit/actor/origin/charge/run/award checks stay intact.
  Extra SB/CS/WP/Balk/PO/A units or optional episode facts are outside this proof.

Whole-chance NoMisplay remains mandatory for this and every preceding event;
missing Error/PB alone is not review. An observed half origin and legal ending
are still mandatory. This is not general walk/HBP history, actual-touch or
occurrence reconstruction, misplay ER reconstruction or complete boxes.

## Observable checks fixed before implementation

1. Both halves, all eight genuinely observed masks and 0–2 outs, Walk/HBP,
   explicit/omitted unforced holds and movement-order permutations. Establish
   occupied bases with observed reviewed BB/WP events; include a positive HR
   before the award and finish with a clearing HR and remaining K-outs.
2. Literal independent R/ER, batting PA/AB/H/BB/HBP and current-pitcher
   BF/AB/H/BB/outs remain correct; one loaded award contributes one forced Run,
   not a run per movement. Roundtrip every transition through the codec.
3. Unknown award/earlier review, omitted observed origin, incomplete half,
   missing award classification and legal excess/unforced advancement keep
   positive charged ER unknown. Retain legal partial logs, not new vetoes.
4. Retain generic contact, non-station hit, error/PB, carried responsibility and
   cumulative-uncertainty controls; no later award repairs earlier uncertainty.
5. Full Rust/Python/fmt/strict all-target Clippy and existing native/audit,
   actual-baseline/2,000-game/100,000-draw/CLI checks. Keep engine/model/seeded
   transitions identical; newly proven ER display changes require exact literal
   controls, not weakened assertions or calibration/threshold retuning.

No DTO/codec, canonical rule, probability, policy, data or UI change is proposed.
This cannot complete E-03/G1. Live evidence and status belong only in
[PROGRESS.md](../PROGRESS.md).
