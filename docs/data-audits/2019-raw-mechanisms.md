# 2019 raw-mechanism lexical inventory

## Contract and provenance

F-01 offline follow-up to the [accepted factual supplement](2019-outcome-details.md),
not a legal-event parser, fitted model or G1/G2 signoff. Acceptance was recorded in
[PROGRESS](../PROGRESS.md) before implementation. No new Parquet or source-data
downloads, engine/UI/RNG/rating changes, delegation or public distribution.

Input: the separate222,047-event/2,429-game supplement, SHA256
`554632467416eef202b7a55cadad2d09e5355ce98c83868b26b5267f98852b82`, snapshot857,
pinned baseball.computer source `b449f7bec49f852c27803e096303a4c78ac661b9`.
The [Retrosheet notation guide](https://www.retrosheet.org/eventfile.htm) was read
in full; an exact64,340-byte HTML copy is retained externally, SHA256
`6006206d4258829935e53c5ce58e95d3ad9fa7f00947d120a5121f998fdb5e7c`.
It defines base plays, `/` modifiers, `.` advance sections, `;` advances and
compound `+` events. Advance text is listed from third base through the batter:
**its textual order is not physical retirement/throw/clock order**.

[`2019-raw-mechanisms.sql`](../../audits/2019-raw-mechanisms.sql) rejects missing,
duplicate, NULL, orphan or conflicting event/child grains; independently aggregates
flags, fielding plays and runners before joining. It exports literal raw plays,
supplied result/out/run/contact/foul-angle fields, ordered source labels and
runner/fielding records, including no-play history. No derived contact retagging.
[`raw_mechanisms.py`](../../audits/raw_mechanisms.py) splits only outer delimiters,
preserves parenthesized annotations, recognizes documented **symbol families**,
and retains structural unknowns with full keys/raw text/reasons. It neither
interprets every advance/modifier nor resolves catch/force/tag/appeal timing.
Unknown roots are not illegal events. An `X` with an error may leave a runner safe;
actual supplied `is_out` and credit units, not that character, control this audit.

## Observed coverage and independently checked controls

External `/tmp/bbsim-raw-mechanisms/events.csv`:84,043,184 bytes,
SHA256 `40c08edc432ed8480e07a660b6896236316c9cb3235ffae6acf0ce9c9d21c199`.
Initial versioned `report.json`:461,680 bytes,
SHA256 `fc7bddcf6f19b6b672717e43da510911e13afbb70509c57c3eb93d5a3f469326`.
Raw data/artifacts remain outside Git; temporary local files are not remotely durable.

- 192,025 real events =186,517 PA +5,508 non-PA;30,022 no-play events separately retained.
- 130,270 outs/PO;43,968 assists;2,836 errors;23,467 runs, unchanged.
- 196,161 raw runner rows =182,718 real +13,443 no-play;9,522 supplied runner-out units.
- 177,670 fielding records:130,270 Putout,44,008 Assist,2,836 Error,**556 FieldersChoice**.
  Forty repeated assists are deduplicated by position within the pinned source's
  Putout/Error-delimited segment. FieldersChoice is a supplied non-PO/A/E record,
  not an invented error/out or a segment boundary. These are not all physical throws.
- 10,548 sparse flag rows, preserved literally.605 real events have multiple outer
  base atoms separated by `+` or `;`; nested annotation `+` does not count as a
  separate compound PA. Atom counts are not PA or opportunity counts.
- 222,036 rows split structurally;11 remain named `empty_outer_segment` records.
  All11 are supplied interference plays containing `C/E2/` (sometimes followed
  by advances). They are retained without rejection, invented interpretation or
  threshold adjustment. No unsupported root symbols among structurally split rows
  does **not** establish complete semantic parsing or universal cause coverage.

Eight independently queried native tables,11 scalar controls and five prior census
partitions (result/out multiplicity/raw trajectory/raw angle/PO-A-E units) match
exactly. Keyed source examples cover K0/K2, zero-out error, TP, appeal/infield-fly/
NDP/obstruction/passing and literal throw-target notation; seven exceptional
Foul/FoulLine cases match direct source rows. Comparisons are separate-path checks
on the **same source**, not independent official scoring/clock evidence.
Full conditional cells and keyed examples are in the external JSON/CSV; no fitted rates.

## Specific signals and their limits

| Supplied notation/label | Observed units | What is not proved |
| --- | ---: | --- |
| `/GDP` / GroundBallDoublePlay |3,463 | Universal runner retirement clock/order |
| `/DP` / UnspecifiedDoublePlay |571 | Ground versus line/fly cause |
| `/GTP` / GroundBallTriplePlay |3 | Physical sequence from credit order alone |
| `/NDP` / NoDoublePlayCredited |12 | Two outs necessarily earn DP statistics |
| `/FO` / ForceOut |3,734 | Every out's cause or a force-opportunity denominator |
| `/IF` / InfieldFlyRule |182 | Recorded narrow contact detail for every such row |
| `/AP` / AppealPlay |4 | Every appeal is an out or identifies a retirement clock |
| `/OBS` / Obstruction |1 | Complete legal timing and actor responsibilities |
| `/PASS` / RunnerOutPassingAnotherRunner |2 | Universal runner-order evidence |
| ThrowToBase(None) |504 | Every throw, target, attempt or defensive opportunity |

Examples with exact source keys:

- ANA201904040/96 (`1254090096`): `K+PB.B-1`, supplied strikeout with0 outs.
- ANA201904080/25 (`1254091045`): `K+CS2(24)/DP`,2 outs, compound base atoms.
- CIN201909020/24 (`1257677364`): `K/NDP.1X2(24)`,2 outs, no official DP credit.
- ARI201906040/9 (`1254607149`): `5E3/G56.B-2`, supplied InPlayOut with0 outs.
- HOU201905220/19 (`1259705629`): `5(2)4(1)3/GTP`,3 outs, literal runner designators.
- CHN201909150/40 (`1257168655`): `OA/AP.3-H(E1/TH)(UR)(NR)`,0 outs/1 run:
  appeal notation is not itself an out.
- LAN201907200/100 (`1260732850`): `E6/TH1/G6S-`, **no sparse flag row**.
- WAS201904280/108 (`1268883423`): `E1/TH2/FO.1-2`, only a ForceOut sparse label.
  Literal TH1/TH2 survives in raw text but these targets are absent from the sparse
  labels; do not reconstruct it from ThrowToBase(None).

Foul2,593 and FoulLine6,959 remain distinct supplied angles, not universal legal
fair/foul/catch classifications. Missing/Unknown contact stays as supplied.
Numeric fielding and caught-stealing/pickoff/error families do not automatically
imply a caught ball, a successful retirement, all throws or clock order.

## Reproduction and verification

Requires the accepted local supplement/manifest and the existing DuckDB CLI;
no dependency installation/network extension loading. Choose new external paths:

```sh
set -euo pipefail
python3 - <<'PY'
from pathlib import Path
Path('/tmp/raw-mechanism-export.sql').write_text(
    Path('audits/2019-raw-mechanisms.sql').read_text() +
    '\nSELECT * FROM mechanism_export ORDER BY event_key;\n')
PY
duckdb -readonly -bail -batch -init /dev/null \
  -cmd 'SET enable_external_access=FALSE;' -csv -nullvalue '' \
  /tmp/bbsim-outcome-details/2019-outcomes.duckdb \
  -f /tmp/raw-mechanism-export.sql > /tmp/raw-mechanism-events.csv
python3 -m audits.raw_mechanisms /tmp/raw-mechanism-events.csv \
  /tmp/bbsim-outcome-details/manifest.json /tmp/raw-mechanism-report.json
python3 -W error::ResourceWarning -m unittest tests.test_raw_mechanisms -v
```

The CLI validates pinned manifest/full-cohort controls, rejects duplicate/invalid
headers and child JSON/grains, rehashes inputs, records export/query/manifest hashes
and refuses existing output files. Native export must use stdout redirection:
`COPY TO` is intentionally blocked by external-access-disabled mode. That retained
interface failure is not a semantic red or a reason to loosen the native guard.

Witnessed semantic reds cover delimiter nesting/unsupported structure, dropped/
fanout/orphan raw facts, literal symbol recognition, duplicate JSON/type errors,
missing pinned provenance and CLI scope/output guards. Import/file-interface
failures remain separately retained.21 focused tests and the full native CLI
passed in `proc_9203`. Fresh `proc_6d4c` exit0 ran
`set -euo pipefail; bash /tmp/bbsim-raw-mechanisms/repeat.sh; bash /tmp/bbsim-raw-mechanisms/full-checks.sh`:
548 Rust/124 Python tests with ResourceWarnings as errors, fmt/build/strict Clippy,
py_compile, all prior audits/fixtures/box comparisons, native1,350 cases,
2,000 seeded games/100,000 draws, baseline equality,19 scalar/17 result controls
and all prior CSV fields/JSON partitions unchanged. Repeated new CSV/JSON are
byte-identical to the hashes above. Original cache/supplement unchanged, all
non-owned tracked bytes equal HEAD, three tested audit/test files frozen. Complete
722 stdout/210 stderr lines consumed; injected FileNotFoundError/HTTP500 are expected
passing negative tests. Exact logs, controls and reproduction scripts are under
`/tmp/bbsim-raw-mechanisms/`.

This is enough to inventory candidate observed outcome shapes, **not** to claim
causal fielding fidelity, legal interpretation of every row, true opportunity
rates or predictive accuracy. Next modeling work needs its own preregistered
conditional support/unknown policy and unchanged engine accounting obligations.
Public rights, G1/G2, production gates and deferred ER work are unchanged.

Attribution: The information used here was obtained free of charge from and is
copyrighted by Retrosheet. Interested parties may contact Retrosheet at
“www.retrosheet.org”. No public extract or bundled-data permission is claimed.
