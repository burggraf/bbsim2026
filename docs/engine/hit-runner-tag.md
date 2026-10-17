# Supplied safe hit followed by one advancing-runner tag

## Contract and observable acceptance (before implementation)

Bounded prerequisite only, not full-game acceptance. Add optional transition-local
`HitRunnerTagFacts`, using the native supplemental constructor/apply/accessor and
codec pattern, never a second reducer. The typed order asserts that the named
batter safely attained the credited Single/Double/Triple base **before** the tag.
The batter movement ends exactly at that base, never Out. One named before-runner
is tagged advancing toward an explicit Second/Third/Home beyond their before
origin (First→Third/Home is allowed). This is a non-force tag, no error or other
cause. Named distinct eligible defensive thrower and tagger bind exactly one
Assist/Putout. Eligibility does not prove real field positions; the later model
wrapper must bind those. Other runners may hold or advance without going backward;
all before runners and their pitcher responsibilities remain explicit.

No overlapping event-local or transition-local causal proofs. In particular,
exclusive `sole_safe_hit` is absent. Require whole-chance NoMisplay, a PA, one Hit
and one value modifier, ordinary PA/AB/BF accounting and named run/out conservation.
No new RBI/ER/complete-fielding certainty or source inference is introduced.
Missing facts keep the existing hit-value + out UnsupportedRule guard unchanged.

Facts explicitly carry `home_before_tag` identities. For the third out, require
matching `ThirdOutKind::Timing` for the tagged runner, with the exact same set of
home-before-out crossings, even when empty. Vector order is never a clock.
All bottom >=9 winning hit+tag leaves are refused, with or without a third out;
the parent must construct a winning-run stop before any extra throw, not retry a
sampled tag. Ordinary nonwinning game endings remain the canonical engine's job.

Acceptance tests: both halfs, all eight masks and three out counts, all three
hit values and each forward tag target; empty-mask refusal, held runners,
inherited charges, named credits/one out, hitter never retired; explicit third-out
runs/no-run and terminal refusals; missing/wrong/overlapping facts, actor roles,
credits, origins/destinations and supplied-after mismatch; movement/credit/timing
permutation invariance; new-facts codec roundtrip/corruption and old no-facts
byte preservation. Record interface failures separately from semantic reds, then
run focused existing hit-value/safe-hit/walkoff tests, owned-file fmt and targeted
strict Clippy. Raw logs: `/tmp/bbsim-text-game-recovery-v1/hit-tag-core-worker/`.

## Clarified legality boundary (parent-approved before final verification)

Forward attempts may skip intermediate bases in the before/after representation:
First→Second/Third/Home, Second→Third/Home, Third→Home. The typed leaf asserts
legal intervening touches before the attempt, not a force-removal/overslide episode.
Reject the still-forced next base from the before occupancy chain. Require the
tag target strictly beyond the hitter's credited base, since the hitter already
attained that base. Safe other runners finish beyond the hitter; runners behind
the retired runner finish strictly short of the attempted tag base, and runners
ahead finish at or beyond it. Catch-up/passing shapes needing additional runner
chronology remain UnsupportedRule rather than inferred. These conservative
refusals are explicit exhaustive test cells, not missing tests or retries.

## 2026-10-17 — native clock-prefix correction (before implementation)

The supplied-hit projector review exposed an overly permissive native clock:
with a First-origin tag at Home and both Second/Third Home endpoints, it accepted
Second before the tag but Third after it. That contradicts the continuous legal
no-passing episode asserted here. The pre-tag crossing set must be a leading
prefix of the Home endpoints in Third→Second→First origin order. An empty set
and any representation order of a valid set remain legal; neither movement-vector
nor ID sorting order supplies chronology. This restriction applies even below
two outs, when both Home endpoints ultimately score. Fix the positive timing
fixture to score the leading Third runner, and add apply/constructor rejection
for the inconsistent subset in every out/half cell. No-facts behavior and old
wire versions remain unchanged. Historical worker counts below predate this fix;
parent verification is separate.

## 2026-10-17 — native survivor-order correction

Independent review identified a second boundary: Second→Home with Third held
and First tagged toward Third could pass tag-relative and crossing-prefix guards.
Parent reproduced acceptance through native apply and a well-formed BBE31 record
with an exact third-out after-state. Continuous surviving before-runners must
retain First→Second→Third endpoint order, independently of Vec order, including
omitted holds. A shared bounded validator applies here and to the new recovered-K
leaf only; no global old-event reinterpretation, endpoint clipping or extra out.
The review's loaded Third→Home tag example was already excluded by the still-live
force guard; it is retained as a refusal control, not claimed as a reproduced bug.
Initial test-fixture errors and corrected semantic reds remain separate receipts.

## Verification and resume

Implemented only the seven authorized paths; parent owns integration and status
ledger. Interface errors were recorded in `01-interface-errors.log` (including a
test accounting-map indexing mistake corrected before semantic execution).
`02-semantic-reds.log` records five failing behavioral tests against API stubs,
with compilation successful; no stub remains. Initial implementation log `03`
retains the expected first force-cell failure before the separately approved
force/no-passing refusal assertions. Expanded tests are in log `06`.

The new target has eight tests: 432 occupied-mask/out/half/value/forward-target
cells (150 supported, 282 explicitly refused), plus all 18 empty-mask/value/out/
half cells, inherited scoring charges and unknown ER/RBI/fielding coverage,
held runners, pitcher eligibility in DH/non-DH roles, malformed facts/credits,
permutations, terminal rules, constructor mismatch and codec revalidation.
`07-legacy-wire-check.log` compares 144 old no-facts records plus re-encodings
against a separately built ed75ecb source archive: byte-identical, SHA-256
`9fb1c07576be6ee1dc58082a8e2336beca4a389ec8ae8f483258d3474ec0ffe1` for both
hex streams. The reproduction shell/probe and streams are retained beside logs.
Final focused regression (278 tests across 15 integration targets plus the
library target), owned fmt, strict Clippy and scope checks pass in
`09-final-checks-with-credit-assertions.log` (prior run retained as `08`); this is
prerequisite evidence, not full-game acceptance. `final-checks.sh` contains the
exact commands. No files are staged.

Next: independent reviewer checks the seven-path diff; parent integrates the
separate model/projector/loop/narrator with position bindings and a winning-hit
stop before any extra throw. No producer changes, full official ER/RBI/fielding
certainty extension, dependency, commit, push or network activity.
