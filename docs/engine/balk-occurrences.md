# E-03 first balk-occurrence slice

**Accepted bounded checkpoint; publication pending in PROGRESS.** Builds on accepted/pushed `3059fbc` and the [approved accounting matrix](g1-accounting-scope.md). [PROGRESS](../PROGRESS.md) owns current status. This first slice is one ordinary Rule6.02(a) imposed occurrence with exact protected awards beginning at the before-state, not every balk cause, a pitching-motion/count engine, complete boxes or G1 signoff.

## Primary evidence and boundary

Cached 2019 Official Baseball Rules/provenance: [2019 rules](2019-rules.md). Rule6.02(a) lines3094–3096/3168–3180 (printed74/76) requires runners, awards each one base without liability, permits play to proceed without reference to the balk under specified batter/runner progress, and preserves wild-throw beyond-award risk and the missed-base appeal minimum-advance exception. Rule5.06(b)(3)(A),1217–1219, distinguishes runners from the batter;5.06(c),1423–1434, does not erase legal advances from acts while alive. Final position, actual reach, legal touch, entitlement and retirement location/order are distinct facts.

Cause matters:6.01(g),2933–2938, expressly combines pitcher balk, batter-first interference award and dead ball.6.02(d)(2)–(4),3280–3294, distinguishes accepted-play/election from automatic ball plus balk with runners. Thus **no universal imposed-balk/non-PA rule or Balk/PA/WP/PB exclusivity** is justified. The first supported ordinary exact-award shape has no PA; other causes remain required unfinished prerequisites.

Scoring:9.02(c)(15),4322, counts pitching balks;9.07 opening4654–4658 excludes balk-assisted advances from SB;9.12(f),5069–5071, forbids Error for advancement as a result of a balk, not every separate chance in an event.9.13 requires legal delivery for WP/PB; a wild throw or bare Balk label is not legal-delivery evidence.9.16(a),5190–5201, allows balks to contribute to ER but does not establish complete reconstruction. This slice **does not extend clean-ER proof**.

## Parent-selected API / wire

- `CreditKind::Balk`, appended scorer tag24; `PitcherLine.credited_balks: u32`, supplied validated units for the actual current pitcher. Zero is credited-only, not certified official zero.
- `Event.balk: Option<BalkFacts>`, appended after `single_delivery`. `None` preserves valid old partial labels/credits; a new positive Balk credit needs sufficient facts. Internal codec `BBE16` preserves earlier tags and rejects earlier prefixes, including15.
- `BalkFacts { cause: BalkCause, disposition: BalkDisposition, resolution: BalkResolution }` scopes **one occurrence and all supplied Balk units in the event**; it is not a complete PA/game history certificate. No redundant pitcher field: canonical participants/current before-defense pitcher own attribution.
- `BalkCause::{Unknown, PitcherAction602a, SqueezeOrStealHomeInterference601g, PitchingProhibition602d}`. These causes must not be substituted for each other. Wire tags0–3 in this order.
- `BalkDisposition::{Unknown, PenaltyApplied, PlayProceedsWithoutReference}`; wire0–2. An ordinary disregarded occurrence contributes no Balk unit, but its actual PA is not nullified. The enum alone cannot certify minimum progress or legal delivery.
- `BalkResolution::{Unknown, ExactProtectedAwards}`; wire0–1. ExactProtectedAwards explicitly asserts that the ordinary imposed occurrence begins at the before-state, all before-base runners' movements are the protected awards, with no additional movement/retirement/personnel sequence to resolve. It is **not** inferred from endpoints, a fielding-review assertion or a completeness flag. Compute named destinations from before origins, not supplied floors.

## First supported shape and known/unknown treatment

Ordinary602a + PenaltyApplied + ExactProtectedAwards requires:

1. Nonempty before bases; runner-only `Balk`, no Plate movement, PA credit, cursor/roster change, retirement or third-out metadata.
2. Exactly one movement for every before-base runner: First→Second, Second→Third, Third→Home. Preserve runner identity and carried responsibility; ordinary canonical conservation/scoring validation remains authoritative.
3. Exactly one supplied Balk unit to actual current pitcher, independent of runner count. Missing/duplicate/wrong actor is IllegalCredit. R and charged R use named legal scorers/carried pitcher responsibility, not current Balk attribution.
4. No phantom pitcher batting row, no PA/BF increment. Unknown or NoMisplay fielding review may accompany a clean exact-award event; neither establishes complete official boxes or positive ER proof.

Definite credit/role/count/review contradictions are checked before missing disposition, cause, pitch compatibility or inherited charge uncertainty. In a typed one-occurrence record, duplicate Balk units are contradictory; without that record, summarized multiple legal occurrences are unrepresented, not a global one-unit-per-event rule. Ordinary explicit PlayProceedsWithoutReference plus its positive Balk unit is IllegalCredit even if progress history is missing.

Legacy `None` without positive Balk stays partial. Unknown cause/disposition/resolution without positive qualification can stay a valid partial view where no coexisting asserted branch requires occurrence binding; omitting the Balk unit does not remove the supplied occurrence. Positive Balk with missing/unknown facts is UnsupportedRule after known contradictions. Declared nonordinary causes, ordinary play-proceeds without minimum-progress witnesses, and richer imposed resolution remain UnsupportedRule until sufficient context is represented; no implicit official zero or new exclusion.

An ExactProtectedAwards witness contradicted by extra/held/retired/missing runner effects is IllegalCredit **for that asserted witness**, not a rule banning such legal balk histories. With unknown/richer resolution those histories remain unsupported. A same-award SB claim contradicts its explicit balk cause; richer independent steal chronology is not globally forbidden. Unknown Error chance/cause, coexisting pitch-record/positive WP/PB, or supplied independent SB without cause/legality/time/arrival binding is unsupported rather than universal incompatibility, even with no Balk unit and Unknown cause/disposition. Final retirement cannot disprove an earlier completed steal. A named actual batter's automatic K (before-first occupied below two outs) or matching explicit before-first retirement cannot receive a completed SB: that definite contradiction precedes balk uncertainty, unlike an eligible/after-first retirement. Known NoMisplay versus Error/PB/interference, including explicit601g cause, remains illegal.

## Observable checks selected before implementation

- Seven occupied masks × three outs × both halves × both fielding reviews and movement/credit permutations: exact awards, conservation, correct score, one Balk, no PA/BF/cursor change, no phantom pitcher batter row.
- Inherited scorer charge/current Balk split; separate contiguous occurrences accumulate without deduplication.
- Missing facts/unknown cause, disposition or resolution, legacy partial labels, required-unit omission, duplicate/wrong actor, empty origins, wrong event context and forged exact-award effects through reducer/constructor; preserve caller state.
- Known actor/count/review/ordinary-disregarded-positive contradictions before unknown history;601g+NoMisplay contradiction despite missing interference credit. Other cause/election combinations must not acquire blanket non-PA or incompatibility bans.
- Unknown-resolution extra advance/Out lawful-history controls remain UnsupportedRule, contrasted with invalid exact witness. No inference of actual progress/touch/order from final endpoints or vector order.
- Same-award SB contradiction; separate/missing chance and pitch-legality/cause-binding uncertainty. Existing WP/PB and legacy None cases unchanged.
- Well-formed uniquely selected codec credit/fact forgeries reach the same semantic validator; earlier prefixes/tags preserve policy without weakened assertions.
- No ER/official completeness extension. All applicable Rust/fmt/Clippy/Python/native/comparator/doc/diff checks, fresh substantive review and final unchanged checks before bounded acceptance/publication.

## Bounded acceptance evidence

Parent consumed the full retained Sol-high follow-up `b170de13-6c85-4d40-b936-ce33f133238a`: **OK with notes; no issues found**, F1–F3 resolved after ten independently reproduced substantive regression reds. Original BLOCK is preserved. Final unchanged `proc_d129`, exact `bash /tmp/bbsim-e03-balk-final-checks.sh`, passed **251 Rust (31 balk)/59 Python/native1,350/27 ER/54 boxes/fmt/strict Clippy/177 prose links/diff**, with all24 source/test hashes unchanged before and after. Review inspected actual source/primary rules/supplied logs, not independently executed or hash-bound checks. Parent accepts only this ordinary imposed exact-award occurrence/counter and shared guards. Shared-upstream comparisons are not historical engine balk reconstruction or calibration. Commands/raw evidence/publication status remain in PROGRESS; no E-03 completion/G1 signoff.

## Required follow-on work (not exclusions)

Ordinary play-proceeds progress witnesses (including eligible uncaught K/other lawful first reaches), missed-base appeal exception, beyond-award wild throws/protection/location/additional entitlements and terminal order;601g interference with attempting-score/defender/action/batter award facts;602d election/automatic-ball/PA completion; cause-bound error/steal/pitch interactions; richer WP/PB/SB/K chronology, Rule9.16 reconstruction and the matrix's complete-field/coverage/literal-box evidence. No executor handoff or implementation authority came from the advisor; parent owns this interface and TDD.
