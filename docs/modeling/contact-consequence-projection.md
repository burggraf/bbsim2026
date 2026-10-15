# P-01-contact-consequence-projection — supplied clean contact decisions

## Preregistered corrective-preview scope — 2026-10-15

D-20/D-21 continue bounded common-mechanism work while managerial integration
remains NO-GO. This is a standalone construction prerequisite, not P-01 completion
or permission to bypass E-03/G1. [Live status](../PROGRESS.md) is the only ledger.
Force/relay functional checkpoint `63410b4ed75d3de458f06592cd3a4dc1d2457802` and
handoff `ec415e90a24ee7c9374e335596e9a0c87ba4796d` are verified locally. The latter
is this task's base;246 HEAD/index/work blobs,154 tested sources,clean main ahead12,
origin804a72d unchanged. No tests or implementation for this task started yet.

Exactly four owned paths:

- `docs/PROGRESS.md`
- `docs/modeling/contact-consequence-projection.md`
- `engine/examples/support/contact_consequence.rs`
- `engine/tests/contact_consequence_projection.rs`

All245 other existing paths, including ALL154 executable/test sources, engine
contexts/codec/accounting and producer/CLI/UI, remain byte-identical. Expected
final249 blobs/156 executable-test sources. Parent sole writer; no delegation,
worktree, dependencies, network/source transfer, fitting, threshold selection,
new RNG, manager integration, release or push. Preserve this exact original
protocol prefix, raw failures, manifests, provenance and source snapshot bindings.
External evidence root: `/tmp/bbsim-contact-consequence-projection/`.

## Actual flow and bounded interface

Existing standalone `runner_only::project(before,decision)` constructs one event
and delegates validation/after-state to canonical apply. `conditional_hit` handles
supplied hit outcomes/words; neither currently constructs SF/bunt/force-relay
histories. The engine's three supplied-facts APIs already own validation,
immutable context, fielding/RBI qualification and BBE28/29/30 revalidation.
Read their actual modules, facts/types and caller fixtures before tests/code.

Add a small independent example-support module with explicit version
`supplied-contact-consequence-projection-v1`, a cloned/debuggable/equatable enum
`ContactDecision` with exactly these variants:

- `CaughtSacrificeFly { catch: OrdinaryAirCatchFacts, facts: CaughtSacrificeFlyFacts }`
- `SacrificeBunt(SacrificeBuntThrowFacts)`
- `GroundForceDoublePlay(GroundForceDoublePlayFacts)`

`project(before: &GameState, decision: &ContactDecision) -> Result<EventTransition,String>`
constructs only the variant's supplied clean episode, then calls its EXISTING
canonical supplied-facts apply API. No alternate reducer, after-state mutation,
unchecked context, context-less apply, swallowed error, fallback, clipping,
resampling, relabeling or repaired actor identities. No caller words are consumed
or generated. An enum variant is an explicit modeled assertion of its documented
clean handling, not something inferred from a GIDP/SH/SF label or endpoint.
Unknown handling/retouch/timing/judgment remains unknown and must refuse.

Derive current batter/pitcher, complete before-runner identity/responsibility and
all explicit movements from the snapshot, never invent a missing runner. Bind
supplied names unchanged; contradictory names must not be replaced with convenient
snapshot names. Every before runner appears exactly once, including runners that
make a shape unsupported; do not silently drop an occupied base. Missing/invalid
construction inputs return an error without modifying before or decision.
Canonical validators retain authority over state, fielding eligibility, contact,
credits, third outs, terminality and immutable after-state.

- SF: one caught batter out, before-Third scores, other runners hold; supplied
  ordinary catch and supplementary handling/retouch/timing facts unchanged.
  PA/NoHit/SF/RBI and named run/putout;no AB. Existing catcher-class support only.
- Bunt: before-First to clear Second, optional Third holds; batter out; supplied
  direct thrower/receiver and scorer judgment unchanged. PA/NoHit/SH/PO/A;no AB.
- DP: First runner forced at Second, then batter before First; optional Third
  holds; exact continuous grounded non-bunt ordered retirement facts and reviewed
  empty error interval; supplied initial/pivot/receiver unchanged. PA/AB/NoHit/GIDP,
  two PO/two A, including permitted initial=First-receiver return alias. Before
  one out supplies matching batter-before-First third-out certificate.

All are PA/BF1,one batting-cursor advance; SF one run/out, bunt zero runs/one out,
DP zero runs/two outs. Preserve half cleanup and carried pitcher responsibility.
No extra optional effects, personnel changes, unsupported base advances or
expanded clean-ER certainty. All Bottom>=9 and actual terminal Top episodes refuse
under the established canonical contracts; late nonterminal Top remains supported.
This module does not choose whether an episode happens, derive a rate/opportunity,
assign a defensive position or assert historical scorer/causal certainty.

## Observable acceptance checks fixed before tests or implementation

1. Compile the wished-for public interface first. Preserve compiler/interface
   defects separately. A typed refusing scaffold may enable genuine behavioral
   reds; do not count missing symbols or already-green refusals as semantic reds.
2. Independent state/accounting grid, not an after-state oracle calling apply:
   contexts Top2/Bottom2/Top9/Top65535 × outs0/1 × nine batting cursors × DH/no-DH ×
   scores[0,0]/[2,1]/[1,2]. SF masks4/5/6/7 × three eligible catcher roles (two
   distinct lineup fielders/current pitcher) × all three existing AirContact
   classes. Bunt masks1/5 × distinct nonpitchers/pitcher-thrower/pitcher-receiver.
   DP masks1/5 × distinct/return alias/pitcher-initial/pitcher-pivot/
   pitcher-receiver/pitcher-return alias. Report exact attempts/acceptance/refusals;
   determine terminal Top refusals literally from score/outs/inning, not engine
   ending helpers. No new empirical data measurement or calibrated interpretation.
3. For every accepted frame, check exact PA/AB/BF/SH/SF/GIDP/RBI/R/H/outs and named
   PO/A units, carried runner responsibility, cursor and literal after-state.
   Roundtrip bytes and normal/pretty Debug, preserve supplied immutable facts;
   compare with separately constructed canonical event fixtures. The latter is
   parity, not the independent oracle. Do not infer ER or repair earlier unknowns.
4. Refusal matrix: missing required First/Third, occupied Second for bunt/DP,
   outs2, ended/invalid snapshot/cursor, false identities, foreign/bench/removed/
   offense/DH fielders, prohibited aliases and unknown/contradictory facts. Check
   Bottom innings9/10/65535 with both relevant outs and all three scores for all
   variants, and terminal Top9/65535. Before/decision unchanged on every error;
   never fallback to ordinary contact or erase the supplied unsupported facts.
   Underlying exhaustive late-Bottom primitive tests remain protected/full-run.
5. Short synthetic mixed accepted prefixes use the new adapter plus established
   ordinary PA/runner-only APIs as applicable. Independently tally PA/cursor/outs/
   runs and prove immutable continuity/codec/stat conservation; explicitly label
   supplied sequence, not generated support, fitted frequency or full-game proof.
6. Preserve every prior source byte and seeded/CLI/wire/normal/pretty-Debug
   compatibility. No producer import/call-site change. Freeze all156 sources
   before a fresh full run: offline locked Rust tests/build,fmt,all-target Clippy
   `-D warnings`; `python3 -Werror -m unittest discover -s tests -v`,compilation,
   ALL inherited source/native/property/legacy/four-mode/page/prefix and BBE27–29
   parity checks. Versioned helpers must authenticate unchanged historical rows
   using explicit `(path,digest)` snapshots, never execute old HEAD-bound checkers
   unchanged at this base or waive missing artifacts. Source fixes require new
   freezes/full runs, not a stale green result.
7. Actually inspect the complete four-path diff in nontruncated displays and exact
   final documentary delta. Recheck protocol/source/input/protected/pin/link/tree/
   HEAD/index equality, record bounded parent-author acceptance and verify a local
   checkpoint. No independent review, producer/manager, calibration, E-03/G1/G2/
   G3/full-box or release claim. Preserve full raw logs; state bounded raw-reading
   coverage honestly. Update Resume at checkpoints; continue until input needed.

## Authentication and initial interface/semantic reds — 2026-10-15

`proc_2483` exit0/16s authenticates ec415e9,246 HEAD/index blobs/245 protected,
all154 prior sources/13 inputs/21 baseline rows and18 predecessor artifact pins;
183 closed artifacts/24 recipes/21 authenticated historical bindings remain exact.
Original protocolSHAab5cbd9214f7b8e43dc3d2b373c52f5013a64f7eb56ec2c3de1723986b7541db
preserved under the evidence root before tests. No transfer or source measurement.

`proc_4ee3` exit101/2s: one missing-module interface compiler error, not semantic.
Wished-for tests cover one supplied positive of each variant with literal after
state, immutable decision, retained facts and BBE28–30 roundtrip. A typed refusing
scaffold, with no accepted construction, enables `proc_9e42` exit101/5s: compiles
without warnings;three methods fail at explicit scaffold Err unwrap,0 pass.
Their later assertions are unreachable in red and not claimed as exercised.
`interface-test-v1.rs`, `scaffold-v1.rs`, `interface-v1.*`, `semantic-v1.*` and both
result JSONs retain exact original bytes, failures and hashes. Grid/refusal/separate
canonical fixtures/mixed-prefix tests, minimal implementation, frozen full checks,
complete inspection, acceptance and checkpoint remain pending. Earlier681/424
full verification certifies only the predecessor's unchanged executable bytes,
not this unimplemented new projector.

## Expanded TDD and first minimal construction — 2026-10-15

Expanded v1 `proc_0e39` exit101/7s compiles:4 positive/grid failures at scaffold Err,
1 already-green refusal method. V2 `proc_e95a` exit101/6s adds separate explicit
canonical event fixtures and seven-frame mixed-prefix test:5 genuine reds/1 green
refusal. Later grid/prefix assertions remain unreachable in red. Preserve original
expanded tests/scaffold/raw streams and result hashes. Canonical parity compares
only cloned event vectors as multisets; no source or raw wire/log value is changed.
The literal after-state and accounting oracle does not call canonical helpers.

Minimal builder now uses actual queued batter/current pitcher and complete carried
runner moves, passing every supplied actor/judgment/handling fact unchanged into
one existing canonical supplied-facts API. No after-state edit, context-less apply,
RNG/fallback, prior-source/engine/codec/producer modification or fabricated runner.
`proc_bef2` exit101/18s:all6 tests pass9.71s, then strict Clippy reports test-only
`needless_late_init`. Original module/test/raw outputs saved. Test-local expression
initialization changes no assertions or application behavior; `proc_0fc7` exit0/17s
freshly passes all6 methods8.19s,strict all-target Clippy `-D warnings` and fmt.
Actual grid23328 attempts (15552 SF/2592 bunt/5184 DP),22896 accepted and432 terminal
Top refusals; literal state/stat checks, retained facts/named PO/A/canonical fixture
parity and BBE28–30 roundtrip all pass. Supplied seven-frame SF/bunt/safe steal/K/
walk/DP/K prefix totals PA6/BF6/outs6/run1,continuous snapshots and runner-only
unchanged cursors. This is not generated support, probability fitting or full-game
validation. Protected245 paths and154 prior sources remain byte-identical.

`proc_7d04` exit0/1s prepares156-source freeze/28 recipes/13 inputs/21 baselines,
21 historical bindings and exact inherited-driver reconstruction under two declared
substitutions (fresh output directory/current HEAD-bound checker). Original24
recipe rows and154-source manifest are retained unchanged; no inherited command
or assertion altered. `proc_a262` exit0/360s runs the full driver from the beginning:
`bash /tmp/bbsim-contact-consequence-projection/full-v1/full-checks.sh` with raw
streams `/tmp/bbsim-contact-consequence-projection/full-v1.{stdout,stderr}`.

## Frozen full verification and complete scoped inspection — 2026-10-15

687 Rust tests/43 zero-failure summaries;424 Werror Python31.197s plus two inherited
3-control preparation suites0.003s each pass. Offline locked build/tests/fmt/strict
all-target Clippy/Python compilation and all inherited source/native/property/
legacy/four-mode/page/prefix checks pass. No source/test/assertion change after
freeze. `proc_955e` exit0/1s reauthenticates156 sources/245 protected/13 inputs/
28 recipes,requires exact Python summary counts424/3/3 and pins163 closed outputs.
Historical predecessor helper's failed one-summary assumption stays preserved;
this new helper correctly prerequires all three summaries,not a waived failure.
Rawstdout129672 bytes SHA c0b19cbbc1683281ecd895138c17c467a1ba185b208fff97cad782a66ee99bfb;
stderr98609 bytes SHA0a45ad5b638135664419fd768c591ea8bcbfa9dcbb22a80b653387fd836eb3c4.
`full-result-v1.json` SHA1db48184a75af065d715d3957ef6c1b35935aa0e6f8e749a46d3f3b0b7b4a0d9;
`completed-v1.sha256` retains the closed artifacts and raw history,not future checks.

Fresh exact bytes/normal/pretty Debug parity:

- BBE27:1152 records/3456 payloads/10238352 bytes,
  SHA e6edb91e97d0042e3123fc4b0c9248e6554ad37288a17850f19219853f7db189.
- BBE28:3456 records/10368 payloads/33558912 bytes,
  SHA d331e6f9c10df51269dd95a6975f9e615faaf8246892d96c488c2169c42a690f.
- BBE29:1728 records/5184 payloads/15647904 bytes,
  SHA be54e5b2058b16698449529f951239be66e59b157789988cfdfded9301ecafe7.

Unchanged original callers/additive export checks pass; BBE28/29 nine methods per
baseline/current library.384 repeated mixed prefixes exact (UniformLegal13331/
4165 runner-only,PooledMask13332/4158,BaseOut13352/4189);16 four-mode records/four
live raw pages/288 legacy records exact. These synthetic parity checks do not
establish empirical mechanism calibration, independent review or manager readiness.

`proc_0cc6` exit0/1s prepares temporary-index tree0f9aa487/83486-byte four-path diff
SHA89766c6ad8f876c9dca6512053395a7b79a8e21f9d220f1336c06ff8c7303e8d;
actual60 complete small views000–059 read/receipted at `inspection-v1/read.json`.
No source/test issue found. Eight raw beginning/end views actually read/receipted
at `raw-reading-v1/read.json`; full raw logs preserved,not all human-read.
Parent-author conclusion: supplied canonical-only construction satisfies the
bounded frozen behavior contract. Exact final documentary delta/stage/pins/links/
tree/local checkpoint and postverification still pending. No independent review,
producer/manager integration, source transfer, calibration or gate approval.

## Staging documentary whitespace failure retained — 2026-10-15

`proc_dba1` exit1/9s authenticates the four-path scope/full results/156 sources,
prepares exact accepted-v1 tree and parent-author receipt,then staged
`git diff --cached --check` exit2 flags this new report's blank line at EOF221.
No commit attempted;HEADec415e9 and exact staged tree verified. Preserve original
helper/raw outputs/acceptance/diff under `failed-staging-v1/`,not a source/test
failure or full-run retry. Unstage only the four owned paths after exact capture;
working bytes preserved. Remove the extra final blank line and record this failure;
no application/test/assertion/input change. Reprepare/read versioned final-doc
v2 delta,then repeat private-index AND actual-index whitespace gates before commit.
Original v1 delta/eight actual-reading views/receipt and all full-run pins stay
unchanged. Postcommit verification is still pending;no gate or producer approval.
