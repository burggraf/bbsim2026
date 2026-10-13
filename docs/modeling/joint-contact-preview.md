# Experimental joint contact-out preview — preregistration

## Scope and dependency boundary

Selected after the accepted ordinary-air prerequisite (`315de0b`), before new
cohort measurements, probability fitting or generator changes. This is a
bounded **P-01-preview** follow-up under the experimental sequencing exception,
not A-02 range/arm modeling or production P-01/P-02/G1 completion. The
[observed support map](observed-outcome-support.md) and
[air contract](../engine/ordinary-air-catch.md) delimit what can be represented
and what source labels cannot certify. PROGRESS owns current status.

The observable goal is an **opt-in**, versioned Rust-generated game containing
ordinary modeled ground and air retirements instead of only one fixed ground
throw. Preserve the current default CLI, baseline, RNG consumption and viewer
behavior. No real players/ratings, manager changes, source imports, desktop
framework, preview restart, downloads or publication. No tag-ups, sacrifice
rules, DP, automatic infield-fly penalty or deferred ER reconstruction.

## Data first: disjoint joint-count recipe

Before generator implementation, build a reproducible read-only recipe from
the accepted 2019 local inputs. Retain the original 185,703 included-PA universe
and all nine baseline category counts/exclusions exactly. Partition only the
82,245 coarsened contact-out observations into five **source-shape proxy** bins:
ordinary-ground candidate, Fly candidate, LineDrive candidate, PopUp candidate,
and residual/unclassified. Each event belongs to exactly one bin.

Air candidate labels require supplied trajectory, one batter-only retirement,
held before runners, no supplied run/error/assist or conflicting supplied
sacrifice/bunt/infield-fly/interference/compound indicator. Ground candidates
need their separately declared supplied geometry/credit-shape filter. Write
these exact predicates and exclusion precedence before measuring the bins.
Missing trajectory/mandatory fields and known unsupported mechanisms cannot
be silently retagged Ground or turned into direct-catch evidence. Report
residual reasons separately and preserve keyed examples, raw strings, modifiers
and unknowns. Independent child aggregates precede joins; NULL, duplicates,
orphans, grain and conservation guards stay strict.

These counts are **not** historical catch certificates, player catch chances,
range opportunities or denominators for per-fielder errors. Endpoint holds do
not prove an uninterrupted hold; supplied credits do not prove throws or clocks.
The new policy would explicitly assume a synthetic direct held retirement for
candidate bins. Its actors/timing are modeled assumptions in fictional teams,
not inferred historical actors. Residual observations remain visible as an
unclassified-out bin with partial causal/accounting coverage.

## Exact initial count predicates and precedence

Reuse the accepted raw-mechanism and outcome-v2 CSV exports and their existing
validators, not a new remote query or legal parser. Strictly join every real
raw event to one outcome event by key and `(game_id,event_id)`; supplied result,
trajectory, angle, outs and runs must agree. Preserve no-play rows in raw
validation but do not give them counts or statistics in this recipe.

For the contact-out branch only, assign the first applicable residual reason:
(1) result is not InPlayOut; (2) not one out/zero runs; (3) trajectory is missing,
Unknown or outside GroundBall/Fly/LineDrive/PopUp; (4) any supplied flag;
(5) credit rows are not exactly one known-position PO, and either zero Assist
rows for air or one distinct known-position Assist row for ground, with no
other row kinds; (6) listed runner effects/unknown endpoints or identities;
(7) structural/compound/advance/unrecognized literal contact notation;
(8) root digits do not match the supplied credit positions. Anything remaining
is one of the four candidate bins. No numerical history cap or legal rejection
is imposed by this *classification*; all noncandidate events remain residual.

For (6), a listed non-batter runner must match its named before origin and
remain there, not out/scored, with `advanced_on_error_flag=false` and
`is_advance_attempt=false`. A listed batter must match the current batter and
have `is_out=true`, `base_end=Out`, no run. A missing batter row does not become
an invented row: one-out/zero-listed-other-outs remains only a source-shape
proxy. Missing before-runner rows similarly do not certify uninterrupted holds.

For (7), use the existing lossless outer lexer; no separators or advances,
exactly one base atom and exactly one modifier. Require its trajectory prefix
G/F/L/P to agree with the supplied tag, followed only by an optional digit1–9,
zero or more literal L/R/S/M/D/X characters and one optional final +/-:
`[GFLP][1-9]?[LRSMDX]*[-+]?`. This is a
conservative lexical filter, not interpretation of range, depth, fair/foul or
clock time. Unknown modifiers remain residual. Air root is one digit1–9 equal
to the PO position. Ground root is two distinct digits1–9 equal to the sole
Assist then PO positions, without treating their printed order as throw timing.
Keep raw strings/tags/angle/modifiers in keyed examples.

Tests must fix this precedence, order invariance, duplicate/grain/NULL/orphan
rejection, conservative unknowns, every before-mask/out partition and original
nine-count conservation before full-cohort evaluation. Initial owned paths are
`audits/joint_contact.py`, `tests/test_joint_contact.py`, this document and
PROGRESS. No changes to existing validators/exports/engine/preview files.

## Initial observed count artifact — verified counts only

`proc_e032` ran the unchanged preregistered recipe twice after two fresh native,
read-only/external-access-disabled exports. Both raw CSVs match the accepted
raw inventory; both outcome CSVs match accepted outcome-v2 bytes. Both JSON
outputs are byte-identical. Complete process output/empty stderr and all1139
JSON lines, including11 literal keyed examples, were inspected. No filter was
retuned after these measurements.

| Source-shape bin | Events |
| --- | ---: |
| Ordinary-ground candidate | 17,188 |
| Fly candidate | 19,528 |
| LineDrive candidate | 5,379 |
| PopUp candidate | 536 |
| Residual | 39,614 |
| **Total contact outs** | **82,245** |

Residual first reasons: non-InPlayOut2,420; non-single/scoreless4,710;
unknown trajectory981; flags3,322; credit shape3,362; runner effects/unknowns2,493;
literal filter22,326; root/credit mismatch0. These sum39,614. Missing zero-valued
context cells mean zero observed candidates, not zero future probability.

The literal residual includes *ordinary known source notation outside this
conservative recipe*, e.g. `ANA201904040`/38/1254090038 `43/G34+`. It is not a
claim that such a historical event is illegal, that its trajectory is missing,
or that every residual event lacks engine support. Likewise candidate labels
are not caught-ball certificates. These are **source-shape frequencies**, not
player catch rates or a true physical contact-out mixture; a later synthetic
mapping must label its assumptions and preserve the residual explicitly.

Outputs remain outside Git in `/tmp/bbsim-joint-contact/`. SHA256:
raw CSV84,043,184 bytes `40c08edc432ed8480e07a660b6896236316c9cb3235ffae6acf0ce9c9d21c199`;
outcome CSV35,059,942 bytes `ea3fd660ac87deb02c61cddf28977db59f85f9a88f77aaabc0c07ba0f1831d1a`;
JSON29,968 bytes `1573fb845ac24d644ac722cc267b3729b2174fc114ff9f6748e2435502fe29ca`.
It preserves the original185,703 included PAs and nine counts,753 IBB/61
interference exclusions,192,025 real/30,022 no-play rows and independent
130,270 PO/43,968 Assist/2,836 Error totals. Original cache and supplement hashes
are unchanged. Query/recipe/manifest/source digests and attribution are in JSON.

Reproduce with the existing native CSV exports documented in the
[raw-mechanism audit](../data-audits/2019-raw-mechanisms.md) and
[outcome audit](../data-audits/2019-outcomes.md), then use new output paths:

```sh
python3 -Werror -m audits.joint_contact RAW.csv OUTCOME_V2.csv \
  SUPPLEMENT_MANIFEST.json SNAPSHOT_MANIFEST.csv NEW_COUNTS.json
```

47 focused tests passed (13 new). Fresh full run `proc_d9a5` exit0/112s:
567 Rust tests,137 Python tests with all warnings as errors, formatting,
offline/locked build and strict all-target Clippy, compilation, prior audits/
fixtures/external boxes, native1,350 cases, baseline equality,2,000 seeded games,
100,000 draws,19 separate-path scalar controls/17 result cells and all prior
partitions/repeats passed. Four complete parent/current default traces are
byte-identical; both source hashes,165 non-owned paths, both database hashes
and count JSON are unchanged. Complete910 stdout/241 stderr lines were consumed;
FileNotFoundError/HTTP500 are expected negative test cases, not acceptance errors.
Final exact four-path staged/local-commit acceptance belongs to PROGRESS.
No sampling, engine, CLI-default, RNG, UI, fitting/accuracy, licensing or gate
changes are accepted by this artifact.

## Fixed probability and sampling contract

Keep the original nine-category Jeffreys posterior means unchanged:
`p_i = (n_i + 0.5) / (N + 4.5)`. Within the contact-out branch, preregister a
five-category conditional Jeffreys distribution
`q_j = (m_j + 0.5) / (n_contact_out + 2.5)`.
The joint masses are `p_contact_out * q_j`, plus the eight unchanged masses.
This is a normalized hierarchical pooled reconstruction prior, not a fitted
fielder-conversion model. Every modeled legal bin remains positive even with
zero observations; no arbitrary epsilon or retry/rejection sampling.

Use **one existing uniform draw per PA** to select the joint outcome. Do not
sample a hit and then independently roll defense to change it to an out.
Keep the first eight category intervals and the total contact-out interval
unchanged; refine only that interval. Preregister the exact floating-point
boundary construction and vectors before changing the sampler. No extra RNG
draw for a catch actor: fictional eligible before-defense roles are explicit
fixed policy assumptions. A residual out must not receive invented catch or
throw facts merely to complete its fielding/ER coverage.

## Fixed observable acceptance

1. Data recipe: exact original nine counts/exclusions, disjoint five-bin sum
   82,245, full before-mask/out partitions, unknown/exclusion reasons, keyed
   positive/negative examples and independent cohort controls. Byte-identical
   native repeats and unchanged input hashes; no network. Actual numerical bin
   counts are not preregistered or assumed here.
2. Probability tests: sums, positivity with zero counts, all-zero conditional
   prior, no impossible category leakage, exact nine-category marginal
   conservation, version/provenance rejection and fixed boundary/RNG vectors.
   Preserve the old sampler/default outputs byte-for-byte.
3. All five modeled bins in every base/out/half frame: legal immutable
   transitions, explicit named facts only for supported modeled causes,
   independent qualified statistics, no repaired prior unknowns. Paired seeds
   preserve base/out/run/batting-order physics where only retirement subtype
   changes. No claim that source candidate geometry certifies those causes.
4. At least 2,000 seeded games: termination, deterministic repeat, full codec
   roundtrip/continuity and available-stat conservation. For 100,000 independent
   draws, each joint category count must lie within six binomial standard
   deviations plus one count of its fixed posterior expectation. These are
   implementation diagnostics, not calibration/accuracy acceptance.
5. Existing full Rust/Python/native/audit/baseline checks, formatting/build/
   strict Clippy, exact scoped ownership/frozen/staged checks and prose links.
   Keep failures/seeds/raw logs. No retuning counts, smoothing or thresholds to
   pass. Local-only verified checkpoints; no gate waiver or independent-review
   claim. Viewer/radio expansion, if needed, requires its own bounded scope and
   browser evidence; it is not included in the initial count/sampler slice.

## Opt-in sampler/policy preregistration (after count checkpoint `c7093c2`)

This next slice has **six** owned paths only: this document, PROGRESS,
`engine/examples/support/joint_contact.rs` (new),
`engine/tests/joint_contact_preview.rs` (new),
`engine/examples/support/preview.rs` (shared game-loop extraction only), and
`engine/examples/generated_game.rs` (explicit opt-in dispatch only). No canonical
engine, count recipe, old tests, CLI parser, diagnostics, commentary or viewer
edits. Preregister first; no behavior is implemented by this section.

Input: a small external `key,value` CSV projected from the accepted JSON, not
another dataset transfer. Exact metadata keys: `format=joint-proxy-counts-v1`,
`cohort_format=2019-joint-contact-proxies-v1`, `catalog_sha256`, `source_commit`,
`snapshot_sha256`, `supplement_sha256`, `recipe_sha256`, `artifact_sha256`,
`included_pa`, `excluded_ibb`, `excluded_interference`, `attribution`.
Pin catalog/source/snapshot/supplement/artifact digests to the count artifact
above and recipe SHA `174e9b959f59359f9fa86c7881dbbf9aefd5981f9376d08b60b975126ff4afc3`;
retain the exact Retrosheet attribution. Include original nine
counts, `included_pa=185703`, exclusions753/61 and five bins in
Ground/Fly/LineDrive/PopUp/Residual order. Version1's CLI loader accepts exactly
this audited pilot vector/provenance, rejects missing/extra/duplicate/malformed
fields or changed counts, and verifies agreement with the supplied legacy
baseline. Numeric CSV values must be nonempty ASCII decimal digits, not signs,
whitespace, floats or exponent notation. Synthetic unit-test constructors may use other nonnegative vectors
whose five-bin sum equals their baseline contact-out count; checked arithmetic
and the existing baseline's numerical support limit apply. No dependencies or
handwritten general JSON parser. Projection must check the JSON digest before
exclusive creation and verify the resulting CSV/roundtrip; keep it outside Git.

Fix versions `pooled-jeffreys-joint-proxy-preview-v1` and
`station-held-joint-proxy-preview-v1`; RNG remains splitmix64-v1. Opt in only via
`--contact-counts PATH` on the generated CLI, with the existing baseline still
required. A wrapper strips that one new option before calling the unchanged
old CLI parser; duplicates/missing values fail. With the flag absent, all old
help/errors/banner, versions, event/wire/debug/trace bodies and RNG use stay
unchanged. The opt-in banner explicitly says source-shape mixture/synthetic
held retirements, residual partial coverage, not calibration. Keep Next/autoplay/
trace behavior; no preview restart or browser-owned outcome code.

**Binary64 construction fixed before new sampler tests/code:** copy the old
nine probability array unchanged. Add its first eight probabilities left-to-
right exactly as the old sampler does; call this last prefix `s`. Compute
`q_j=(m_j+0.5)/(n_contact_out+2.5)` and their first four cumulative sums in fixed
bin order. Append edges `s+(1-s)*cumulative_q`, using multiply then add (no FMA),
then exact1. Compare one `u=(next_u64()>>11)/2^53` with edges using strict `<`;
edge equality belongs to the next bin. No second draw, normalization loop,
retry or conversion of a selected hit. The reported mathematical joint masses
remain `p_contact_out*q`; actual tail width is the legacy sampler's `1-s`,
whose last-bin rounding fallback is preserved. Never perturb the first eight
intervals. Reject nonfinite/out-of-domain explicit test draws and unsupported
vectors with collapsed/non-increasing boundaries, rather than invent an epsilon
or silently drop a legal bin. Probability/interval positivity is strictly `>0`.
Fixed numerical assertion tolerance is32 binary64 epsilons for sums/marginal
mass, not a probability floor.

Independent arithmetic vectors were recorded before implementation in
`/tmp/bbsim-joint-preview/preregistered-boundaries.json`. Audited13 CDF edges,
in order (hex binary64 bits):

```text
3fcd842faa8b150b 3fd3fa095bd4e642 3fd4a91e5c6cd28e 3fdd9a54a2f15f90
3fe0458294e445f7 3fe068290d556a82 3fe193168140df9b 3fe1d3f485c334cd
3fe4ca29925f5f41 3fe827970f269a30 3fe914e30924145f 3fe92c8d7590da2c
3ff0000000000000
```

At each edge test predecessor/exact/successor binary64 values and0/largest
representable value below1. Test the all-zero conditional prior (a nonempty
baseline with zero observed contact outs), single concentrated bins and extreme
unsupported vectors. For seeds1/2/3/99, the selected original category and next
RNG word must agree with the old sampler, including contact's rounding tail.

**Synthetic policy:** first eight categories reuse the existing event producer
unchanged. Ground candidate reuses its explicit ground throw. Fly/LineDrive/
PopUp reuse contact-out geometry but remove ground facts/Assist and bind explicit
air facts, exact PO to an eligible named before-defense actor and NoMisplay.
Choose deterministic eligible defender ordinals6/7/2 respectively from before
batting-order fielders excluding DH, then pitcher once (de-duplicate); these are
fictional actor assignments, **not positions/range/arm estimates or inferred
historical actors**. No actor RNG or separate fielding conversion. Residual
keeps the modeled one batter-before-First retirement/held-runner geometry and
PA/AB/NoHit, but removes ground/air facts and PO/Assist and uses Unknown review.
Do not invent qualified defense or ER to fill its box. All types conserve the
same physical score/base/out/batting cursor, including third-out/terminal frames.

Extract the old initialized-state/apply/10,000-event resource-ceiling loop once
into a shared example helper accepting one producer callback; the legacy game
calls its existing sampler/producer, the opt-in game its joint sampler/policy.
Do not replay an already sampled legacy game with another RNG pass, duplicate
the full-game implementation, add retries or weaken the ceiling. Retain old
seed/error behavior and exact parent/default trace/wire controls.

Acceptance: genuine interface/semantic reds before behavior; pinned CSV/header/
provenance/cohort rejection; exact boundary/RNG vectors; all5×8×3×2 policy frames
plus DH/no-DH/actor/malformed-state controls; independent fielding/ER unknowns
and no repaired prior actor uncertainty. At least2,000 paired seeded full games
must terminate/deterministically repeat, codec-roundtrip/continue and conserve
available statistics while matching legacy physics; confirm all five types
actually occur.100,000 joint draws must satisfy the already fixed six-sigma+one
count diagnostic against joint expectations, without retuning seeds/thresholds.
Run full project/native/audit checks and exact six-path staged/frozen/parent
compatibility/prose-link acceptance. No predictive, production, G1/G2 or rights
claim; counts remain proxies and coarse runner outcomes remain synthetic.

## Reproduce the opt-in experiment locally

Internal offline pilot only; this is not a physical contact model or an MLB
matchup. The default command stays unchanged. The new flag uses an independently
validated logical projection of the accepted aggregate JSON, **not** a hash
claim for arbitrary CSV bytes. The loader pins all26 fields/values and checks
the legacy marginal vector; reordered rows are equivalent. Attribution's site
quotes are literal in this flat key/value grammar (no general CSV escaping).

Given the accepted artifact, make a **new** external file:

```python
from pathlib import Path
import hashlib, json
source = Path('/tmp/bbsim-joint-contact/counts-first.json')
raw = source.read_bytes()
sha = '1573fb845ac24d644ac722cc267b3729b2174fc114ff9f6748e2435502fe29ca'
assert hashlib.sha256(raw).hexdigest() == sha
report = json.loads(raw)
provenance = report['provenance']
fields = dict(format='joint-proxy-counts-v1', cohort_format=report['format'])
for key in ('catalog_sha256', 'source_commit', 'snapshot_sha256',
            'supplement_sha256', 'recipe_sha256'):
    fields[key] = provenance[key]
fields.update(artifact_sha256=sha, included_pa=sum(report['counts'].values()),
              **report['exclusions'], attribution=provenance['attribution'],
              **report['counts'], **report['contact_bins'])
assert len(fields) == 26
assert report['format'] == '2019-joint-contact-proxies-v1'
assert sum(report['contact_bins'].values()) == report['counts']['contact_out'] == 82245
assert all(',' not in str(v) and '\n' not in str(v) and '\r' not in str(v)
           for v in fields.values())
text = 'key,value\n' + ''.join(f'{k},{v}\n' for k, v in fields.items())
assert dict(line.split(',', 1) for line in text.splitlines()[1:]) == {
    k: str(v) for k, v in fields.items()}
assert source.read_bytes() == raw
output = Path('/tmp/joint-contact-counts-new.csv')
with output.open('x') as stream:
    stream.write(text)
assert output.read_text() == text
```

```sh
cargo run --offline --locked --manifest-path engine/Cargo.toml \
  --example generated_game -- --baseline /tmp/bbsim-preview-2019-counts.csv \
  --contact-counts /tmp/joint-contact-counts-new.csv --seed 1 --autoplay --trace
cargo test --offline --locked --manifest-path engine/Cargo.toml \
  --test joint_contact_preview -- --nocapture
```

Model/policy versions and explicit limitations appear in opt-in output; the
existing nine probabilities remain unchanged and five conditional proxy
probabilities/13 CDF edges are reported separately. Generic narration is not
catch-specific; inspect native trace's explicit air facts/Unknown residual
review. Residual PO/A/E/PB and relevant ER coverage remain unknown rather than
being filled from the modeled physical out. No tag-up, sacrifice, multi-out,
real-player, catch-opportunity or range/arm support is added. Browser/server
integration remains out of this slice and previews remain stopped.

Focused parent checks already cover boundary/RNG preservation,480 contact
frames, all DH slots/no-DH pitcher catch, malformed/terminal state rejection,
pinned loader/option boundaries and irreversible uncertainty. The preregistered
2000 paired games and100000 draws are diagnostics of this synthetic experiment,
not held-out accuracy. Fresh full/native/default-byte acceptance and the live
status are recorded only in PROGRESS; do not infer signoff from this contract.

## Initial implementation boundary

Start with the count recipe and tests, not the sampler. Its owned source paths
and exact predicates must be recorded in PROGRESS before implementation.
Later CLI/policy ownership and versions likewise need preregistration before
behavior changes. Existing preview/model/engine contracts remain unchanged
until the corresponding tested slice is explicitly accepted. Do not silently
replace the current default with this experiment.
