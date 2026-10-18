# 2019 dated-roster feasibility probe v1

## Decision

**NO-GO for full-season retrieval or autonomous selection from these inputs.**
This is a small source-feasibility result, not a finding that historical rosters
are impossible to obtain. Keep the verified observational ledger and historical
lineup mode; do not infer eligibility from appearances or repair source gaps.
Live status and the next authorized action belong in [PROGRESS.md](../PROGRESS.md).

The user authorized this probe in interview
`a1b82cd0-baac-4f79-a16f-8d00944f7047`, preferring baseball.computer if it supplies
verified dated availability. Full2019 retrieval was conditional on a passing probe;
that condition was not met. No bulk retrieval was performed.

## baseball.computer checked first

Upstream commit `1195c593898c1bb085a760a0ce85560321a421c6`:

- [stg_rosters.sql](https://github.com/droher/baseball.computer/blob/1195c593898c1bb085a760a0ce85560321a421c6/bc/models/staging/misc/stg_rosters.sql)
  has season/player/team grain, hands and position, but no dated active/IL intervals.
- [synthetic transactions helper](https://github.com/droher/baseball.computer/blob/1195c593898c1bb085a760a0ce85560321a421c6/bc/python_models/synthetic_box_scores/transactions.py)
  consumes Retrosheet team changes, includes the trade date in both teams' windows
  and falls back to proportional allocation. It is not verified daily availability.

These specific sources do not meet the requirement. This is not a claim that every
possible upstream artifact was exhaustively searched. No roster Parquet, Retrosheet
archive or whole DuckLake catalog was downloaded for this probe.

## Bounded MLB API test

The preregistered requests used Yankees, Mets and Toronto samples across opening
and late season, IL/activation and a trade. Fifteen read-only data requests returned
147,784 response-body bytes:12 roster snapshots,2 transaction ranges and1 targeted
person lookup. All HTTP200, untruncated;20s timeout and524,288-byte cap per request.
No retries, installs, credentials or paid sources. Retrieval timestamps, exact URLs,
headers and raw bytes are retained. Body bytes exclude transport/header overhead.

Useful sampled results:

- Historical date filters change membership; Judge's IL/activation and the Mets'
  acquisition of Stroman have matching within-API numeric-ID witnesses.
- All12 roster sets are nonempty, duplicate-free and match requested team/type.
  Same-day statusA subsets of40Man responses match active responses.

Blocking results, retained rather than repaired:

- Stroman(573186) is absent from the Toronto July27 active response before the
  July28 TOR→NYM trade. The reason is unresolved: earlier inactive status, omitted
  history or other source semantics are possibilities. **Not proof of a database
  error**, and not evidence of a complete outgoing active roster.
-22 of125 Yankees transaction records have different `date` and `effectiveDate`.
  One recall has April21 versus June7. Retroactive IL dates account for some
  differences, but there is no verified general interpretation for all cases.
- No roster as-of time, game identifier or pregame/end-of-day semantics was
  returned. Calendar dates alone do not establish game1/game2 availability around
  same-day moves. Roster responses do not echo the requested season/date.
- The40Man responses contain44/50 rows, including60-day IL and reassignment
  statuses. They must not be treated as strict40-man or active eligibility lists.
- `people/592450?hydrate=externalIds` provides no Retrosheet crosswalk. This only
  establishes that this requested hydration did not supply it, not that no exact
  crosswalk exists elsewhere. Names/birth dates are not substituted for stable IDs.

## Evidence and verification

Original private evidence: `/tmp/bbsim-dated-roster-probe-v1/`.
`preregistration.md` precedes player-data requests. `request-manifest-v2.json`
authenticates all15 responses; `transaction-date-semantics.json` retains all22 date
mismatches. Recipes/raw failures/initial manifests are retained without overwrites.
Worker report: subagent `5fbbd0d2-bbec-483d-81fb-a9aadfdfcb3b`, output
`audits/dated-roster-probe-v1.md` in its retained subagent artifacts.

Worker commands: `python3 -B /tmp/bbsim-dated-roster-probe-v1/{probe,identity_lookup,check_evidence}.py`.
These are three distinct exclusive-output commands, not safe-to-overwrite reruns.
The offline checker reported `OFFLINE_EVIDENCE_CHECKS_PASS; SCIENTIFIC_DISPOSITION=NO_GO`.
Parent independently verified58 evidence hashes/lengths, all15 request totals,
the raw outgoing-trade omission,22 date mismatches and missing crosswalk field;
no data requests or completed drivers were rerun. Integrity passes do not turn the
scientific NO-GO into a pass. No repository implementation/tests changed here.

Next candidate, subject to separately approved bounded investigation: game-specific
MLB roster records with timing evidence, plus an exact Retrosheet↔MLB identity
crosswalk. Reuse existing responses first. No silent date convention, inferred
window or full-season fetch is authorized by this failed probe. API access does
not establish redistribution clearance; terms and whole-season completeness remain
unverified. This result changes no G1–G5 gate.
