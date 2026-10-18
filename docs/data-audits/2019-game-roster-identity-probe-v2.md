# 2019 game-specific roster and identity probe v2

## Decision and advance

**Promising source path; no authenticated selectable-personnel or full-season
retrieval GO.** The second bounded investigation is complete under interview
`ebe8c22e-f1c3-49b1-816d-5c9e8335e491`. All20 authorized requests are consumed;
no additional network, bulk import or automatic manager enablement follows.
The user's verified-dated-availability requirement is unchanged.

Unlike the [first daily-roster probe](2019-dated-roster-probe-v1.md), historical
game-specific feeds contain explicit **unused bench and pitcher members**. A
revision-pinned, exact-ID candidate bridge also exists. These are useful findings,
not proof of complete pregame availability or correspondence for every player.

## Scope and provenance

Private evidence: `/tmp/bbsim-game-roster-probe-v2/`; root mode0700. Preregistration
preceded all requests.20 HTTPS GET attempts,2,103,150 response-body bytes, allHTTP200,
no errors, redirects, retries or truncation. Largest response649,946 bytes;
2MiB/response and8MiB total caps. Documentation/property/terms requests count toward
20 conservatively. No all-season schedule or all-time register downloaded.
Python stdlib/socket timeout20s; observed longest request2.766049s. The configured
socket timeout is not a guaranteed total-wall deadline for an arbitrary slow stream.

`request-manifest.json` and each raw/attempt/receipt file retain URLs, UTC times,
headers/status/length/SHA256. `request-recipe.txt` records exact completed commands,
not safe-to-overwrite rerun instructions. Earlier15 responses/59 files remained
unchanged. No repository implementation, dependencies, DBs or servers changed.

## Game-record evidence

Bounded schedule discovery: TOR/CLE2019-07-24 gamePk566914; NYY/KC April20/21
567469/567470. Feeds explicitly identify2019 regular-season game/date/teams.

- Earliest retrieved TOR response:21:00:20UTC, Pre-Game, before23:07 scheduled
  start and first pitch23:09:05.454. Later timecode22:52:10 is Warmup; final is
  July25 01:33:56, Final. Timecodes demonstrably affect historical game state.
- TOR's24 listed pregame members include two bench players who remain unused
  in the final record (Biggio624415, McKinney641856) and ten unused other pitchers.
  Cleveland also includes unused bench/pitchers. These are not participant-only sets.
- Stroman573186 is TOR's listed starting pitcher July24. This does not explain
  the separate July27 daily-roster omission found in v1.
- Judge592450 is listed for NYY before his April20 in-game injury and absent
  before April21, consistent with the retained April21 IL transaction.

Limitations that prohibit treating these as verified complete pregame pools:

- Historical game state coexists with current biography: an early TOR response
  includes Stroman `lastPlayedDate=2025-07-31`, current age and biography active=false,
  alongside game-box statusA. Ignore biography active for historical eligibility.
  This is not proof that every field is a frozen historical publication.
- Asking for20:00 before the first listed21:00:20 timecode returns byte-identical
 21:00:20 data: a forward clamp. A future consumer must check returned time, not
  assume the requested cutoff was respected.
- Maile571912 is absent from TOR's24-member list; a July27 IL placement is retroactive
  to July24. Evidence does not distinguish true unavailability from later
  reconstruction. Do not insist on25 or add him mechanically. McGuire's July27
  recall has effectiveDate2020-09-05, another unresolved date-semantic issue.
- No captured provider definition/independent complete pregame record establishes
  roster-list completeness or historical publication cutoff. Doubleheaders and
  other games remain untested. This does not prove the lists are wrong; their
  suitability for the required complete pregame availability is unverified.

## Exact identity candidate

Wikidata P3541 (MLB ID) and P6976 (Retrosheet ID), queried by exact IDs and reversed,
produce three unique sampled pairs:573186↔strom001,592450↔judga001,624415↔biggc002.
Direct entity JSON agrees; captured revisions/GUIDs/response hashes are retained.
This is not a name or birth-date join and does not require an all-time register.

However, all three Retrosheet statements lack references; MLB statements reference
Wikipedia imports rather than captured official paired-ID records. This authenticates
Wikidata's assertions, not independently verified identity correspondence. Season-wide
coverage/uniqueness and the accepted validation authority remain unresolved.
Wikidata structured data's captured CC0 terms do not establish factual correctness
or rights to MLB/Retrosheet extracts.

## Rights boundary

The captured [MLB notice](https://gdx.mlb.com/components/copyright.txt) states:

> Only individual, non-commercial, non-bulk use of the Materials is permitted and
> any other use of the Materials is prohibited without prior written authorization
> from MLBAM.

The small private probe is not clearance for a full-season import, redistribution
or commercial use. No outreach to providers is authorized. Any broader acquisition
needs its own rights/scope decision; user permission alone does not grant provider
rights. Do not fetch more data under the exhausted20-request budget.

## Checks and handoff

Worker `1b576911-7040-44c5-a63d-099dffe3b47d`: `fetch.py --self-check`,
`analyze.py --self-check`, exclusive `analyze.py` and `verify.py` completed; command
logs, `results.json`, `verification.json` and93-file `evidence-manifest.json` retained.
Parent independently verified all93 sealed files,20request/byte totals, raw pregame
members, forward clamp, mixed biography, unreferenced exact ID statements and terms.
All source/test bytes from the prior722Python/946Rust/4example verification remain
unchanged. No full suite rerun was needed for these documentary-only changes.

Next needed: a permitted dated-roster/availability source and accepted exact identity
validation, including resolution of completeness/retroactive timing. A research-only
fixture from these saved records is feasible but would not implement or certify the
autonomous historical manager. Further acquisition or changing the verified-roster
requirement requires an explicit maintainer decision. Current status is in
[PROGRESS.md](../PROGRESS.md); no G1–G5 signoff or full-season certification.
