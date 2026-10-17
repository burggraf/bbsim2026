# Text-game recovery — observable contract

Authorized by the maintainer on 2026-10-16 after the playable preview exposed an
unresolved contact-out bucket. This is an integrated game-generation task, not
another standalone-helper milestone. Live status belongs in `../PROGRESS.md`.

## Deliverable

A Rust text-game command, reproducible seeds and complete generated game files.
Each event contains:

1. **Before:** both team scores; top/bottom and inning; outs; bases-empty or all
   occupied bases with runner names; current batter and defensive pitcher.
2. **Play:** an explicit mechanism and result, including contact direction and
   named fielders for balls in play; errors and throws where applicable; every
   runner's advance, hold, retirement and scored/uncounted run.
3. **After:** the same state fields, including normalized inning changes or final
   status. The next event's before-state equals this after-state exactly.

No catch-all/residual outcome may become a finished play. No debug-facts fallback,
undefined fielder or location, or retrospective narrative-only invention. Walks,
strikeouts and other non-contact plays must not acquire fictional contact.
Sampling selects a supported mechanism **before** constructing its transition.
A selected hit must not face a second independent conversion-to-out roll.

The canonical Rust reducer remains the only authority for game state. Sampled
mechanism/contact facts must be bound to the corresponding immutable transition,
not maintained by a text renderer's separate game simulation. Explain the precise
claim boundary of any additional model-level facts not represented by the old
canonical schema; do not imply that old validators validate new contact details.

## Major-event coverage to implement and exercise

- Called/swinging strikeouts and uncaught-third-strike outcomes, with eligibility.
- Ordinary/intentional walks; hit by pitch; home runs; singles/doubles/triples.
- Grounders, line drives, flies and pop-ups with legal defensive positions and
  contact directions; infield and outfield outs, including first baseman
  unassisted, pitcher/catcher fielding, force plays and fielder's choice.
- Ground double plays, caught-ball/tag-up consequences and caught-ball double
  plays; sacrifice flies, sacrifice bunts and failed bunt attempts.
- Fielding and throwing errors with resulting runner movements, not a fixed
  error assigned to the shortstop for every chance.
- Holding/advancing/scoring on hits and eligible outs; attempted extra bases and
  runners thrown/tagged out; correct force-versus-timing third-out scoring.
- Steals, caught stealing, pickoffs, wild pitches, passed balls and balks,
  including inning-ending runner events that do not consume a PA/batting slot.
- Legal pitching/personnel changes through a separate minimal manager policy;
  no re-entry or silently fabricated historical availability.
- Regulation endings, unplayed bottom ninth, extra innings and walkoffs,
  including terminal runner-only and multi-effect events under 2019 rules.

This list is an acceptance target, **not a statement that these are implemented**.
Explicitly report any remaining target gap rather than mark the task complete.
Pitch-by-pitch counts/foul balls, injuries/ejections, weather/suspensions/forfeits,
appeals and exotic interference/obstruction rulings are not part of this
PA-plus-runner-event recovery. Do not claim exhaustive Official Baseball Rules
coverage. Existing source interference mass must nevertheless have an explicit
supported treatment or refusal, not disappear into renormalization.

## Model and evidence, fixed before fitting/comparison

Use existing audited 2019 inputs and validated identities/positions where supplied;
retain provenance and seeds. Historical roster availability and empirical contact
geometry must not be fabricated. Explicit neutral/coarse modeled priors are
acceptable initially, identified as generated assumptions rather than observations.
Record probability taxonomy, context eligibility, smoothing/priors and fallback
allocation before parameter estimation or comparison. Unknown source cause is
not evidence for a particular mechanism. No calibration or forecasting claim.

Acceptance is integration, not just a list of helper tests:

- Deterministic tests for every listed mechanism, ineligible contexts, base/out
  combinations and linked/multiple effects; inspect failure before behavior.
- At least 1,000 ordinary seeded complete games: no unresolved plays, illegal
  transitions, lost/duplicated runners, lineup drift or fabricated terminations.
  Preserve failing seeds; a resource cap must report failure, not award a winner.
- Independently reconcile runs, outs, PA, runner-only PA invariance, and fielding
  credits for the mechanisms claimed; disclose incomplete official box accounting.
- Full applicable Rust/Python suites, formatting and strict Clippy; do not weaken
  assertions or replace required suites with new smoke tests.
- Deliver complete ordinary games for preregistered seeds **1, 2 and 3**, without
  editing, cherry-picking or forcing variety. Separate deterministic coverage
  transcripts may demonstrate rare events; label them as constructed tests.
- Publish exact reproduction commands, versions, coverage failures and limits.

No UI work, new dependencies, season/release expansion, public data distribution
or gate signoff is authorized by this recovery. Keep prior uncommitted work and
paused supplied-producer artifacts intact. Make the smallest integrated extension
that satisfies these checks; do not restart the project or build another audit
framework before producing baseball.
