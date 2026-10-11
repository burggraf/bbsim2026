# E-03: ordinary ground throw to first — selected evidence contract

Selected after published PA H checkpoint `960d3b3` / handoff `57ba400`, before
schema or behavior implementation. [Approved matrix](g1-accounting-scope.md) and
[coverage requirements](scoring-coverage.md) remain binding. This is a bounded
main-path prerequisite, not an ordinary-out family exclusion or G1 signoff.

## Actual gap and primary rules

`support/preview.rs::event_for(ContactOut)` chooses a coarse held-runner ground
out: actual batter Plate→Out, one out, zero runs, PA/AtBat/NoHit, named assist
and putout. Its lineup indices select fictional defensive actors; the engine
has no permanent shortstop/first-baseman position map. It must not infer one.

`rules::validate_credit_counts` caps PO by outs; `credit_role_eligible` checks
possible active defenders. Neither binds a supplied PO or A to a particular
retirement or effective throw. `TwoOutContactFacts` binds two retirements and
GIDP context, not this one-out fielding sequence. `clean_play` deliberately
leaves generic contact unknown even with NoMisplay and supplied PO/A.

Cached 2019 Rules 9.09(a)(2) and 9.10(a)(1), including its effective-deflection
comment, bind PO to the fielder catching/tagging the base for the out and A to
the effective contributing throw. Rule 9.10(b) excludes, among other things,
a pitcher's mere delivery and certain misplay-following throws. These are not
licenses to assign an assist from a putout alone or every ball touch. Primary
cache: `/tmp/bbsim-2019-rulebook.txt`, lines4797–4918; PDF SHA256
`d85e0e43cffaff4ed916065d5434b26b0d93d54dbd91630d585eeefdfca29e92`.

## Smallest positive factual shape

Introduce optional event facts for **one ordinary, fair, non-bunt ground ball**
controlled by a named thrower, directly and successfully thrown to a distinct
named receiver who catches it while touching first before the named batter's
first touch. This binds the entire terminal contact sequence: no other fielding
touch, relay, deflection, appeal, automatic out or interference retirement.
Before runners remain at their before bases throughout this contact sequence;
there is no additional actual retirement or scoring on the play.

Preferred representation: `Event.ordinary_ground_throw:
Option<OrdinaryGroundThrowFacts>` with explicit `batter_id`, `thrower_id` and
`receiver_id`. The subtype specifies the method/base/touch order, not an
unchecked complete-box flag. Append this optional field after `balk`, preserve
all existing tags/layout prefixes and use BBE20 with earlier-prefix rejection.
Every historical constructor supplies None; do not add facts to source fixtures.

- Actual generic PA batter/participants, eligible before-defense actors and
  distinct thrower/receiver are mandatory. A pitcher can field/throw or cover
  first when eligible; a DH/bench/removed/offense actor cannot.
- Exactly the batter's Plate→Out movement, one out and zero runs; runners are
  held implicitly or through explicit self-holds preserving identity/charge.
  No personnel change or incompatible contact/delivery/balk fact combination.
  At two before outs, require the actual batter's BeforeFirst third-out record
  with no home-before-out list. Otherwise there is no third-out record.
- For positive qualification, require the existing one PA/AtBat/NoHit
  classifications and matching one receiver PO/one thrower A, without additional
  PO/A units. None of the facts/modifiers adds a second PA/AB/BF/H/out. Enforce
  qualified unit omissions/excess only after the applicable review prerequisites;
  an Unknown earlier-chance review is not proof that every extra supplied assist
  is impossible or that an absent assist is a reviewed official zero.
- Known actor/method/movement/unit contradictions precede unrelated missing
  review/charge uncertainty through reducer, constructor and semantic decode.
  Illegal supplied facts are not repaired, swapped or silently dropped.
- Whole-chance NoMisplay remains required for **positive per-event PO/A
  coverage**; Unknown retains partial supplied-credit accounting and no coverage
  certificate. A represented Error/PB contradicts NoMisplay under existing rules.
  Additional unbound episode/chance facts cannot gain coverage from None fields.
  Legal richer histories use the existing partial representation or remain
  explicitly unsupported, not structurally impossible outcomes.

## Independent contribution, not an official box

For immutable validated events, expose an optional **event-local** qualified
PO/A contribution derived from the named factual sequence, not from counting
supplied PO/A or reading final totals. Some binds PO1 to the receiver and A1 to
the thrower; other before-defense actors have zero **in this qualified event**.
A missing factual sequence or review yields None, not an empty known map.
Existing credited fielding totals remain lower bounds. Other event families
must acquire their own coverage before these contributions can form complete
player/team PO/A cells. No complete-game boolean or official E/PB/RBI/WP/balk
zeroes, no new clean-ER proof, no source position/rating inference.

After canonical/fold proof, the preview may supply these facts for its chosen
ordinary contact-out category only. Do not relabel its K, ROE or coarsened
historical source outcomes. Preserve RNG, probability/model/policy versions,
physical seeded histories and existing partial/source controls.

## Observable acceptance fixed before TDD

1. All eight masks × three before outs × both halves, both held-runner vector
   forms, relevant order permutations, eligible pitcher/no-DH and ordinary
   defender pairs: one named actual PO/A, exact independent contribution,
   continuity/codec/PA/AB/BF/H/out/score conservation and terminal attribution.
2. Qualified PO/A omissions/excess, known wrong/duplicate units, foreign/DH/
   bench/removed actors, wrong named batter, same thrower/receiver, incompatible
   PA classifications and false third-out/touch/run/movement/charge claims:
   exact canonical errors in reducer/constructor/parsed semantic bytes, with
   valid parse controls. Pair review-dependent unit assertions with Unknown
   earlier-chance controls; do not reject a potentially legal extra assist
   merely because this narrow sequence cannot establish its completeness.
3. Missing facts/review, legal generic contact and richer partial histories
   cannot certify coverage. Bare NoHit/AtBat/PO/A/NoMisplay remain insufficient.
   Preserve the existing generic-contact ER unknown test and historical fixtures.
4. Contributions derive from facts rather than supplied counts; actor/opportunity
   scope remains explicit. A physically complete game with omitted field facts
   remains incomplete for PO/A. E/PB and other agreed fields are not inferred.
5. Producer facts bind its selected thrower/receiver without changing physical
   outcomes or sampling; all 1,296 existing contexts and 2,000 synthetic games,
   plus actual-baseline diagnostics and CLI/restart controls must pass freshly.
6. Fresh complete applicable Rust/Python/native/audit checks, raw reds/failures,
   frozen code/seed evidence and complete owned/staged inspection before any
   implementation acceptance. Review scope/authority and remaining G1 proof
   stay explicit in [PROGRESS](../PROGRESS.md).

## Next implementation action

Write interface tests and paired factual/missing-fact/credit-forgery controls
first, distinguish compiler plumbing from semantic reds, then implement only
this contribution family. Do not infer new facts from the current credits.
Status and exact execution evidence belong solely to PROGRESS.
