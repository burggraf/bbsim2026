# Sources and evidence boundaries

Research recorded **2026-10-17**. These are citations and inspection notes, not
vendored datasets, downloaded commercial cards or a license clearance. External
pages can change. Only the GitHub generator inspection below has a pinned source
revision; a live URL is not an immutable data version.

Read [DESIGN-AND-MATH.md](DESIGN-AND-MATH.md) for conclusions and
[README.md](README.md) for the confirmed fully physical/offline requirement.

## Established-game rules and cardmaking

### S01 — Strat-O-Matic official board-game overview

- [How To Play — Board Games](https://www.strat-o-matic.com/board-games/)
- **Evidence:** publisher's public description; page read.
- **Supports:** first die selects batter/pitcher and column; two other d6s are
  summed for the row; some plays require a second check.
- **Does not establish:** proprietary card-generation formulas, a literal 50/50
  division of real baseball causation, or independently measured accuracy/speed.
  Publisher marketing claims are not our benchmark results.

### S02 — Strat-O-Matic official hitter-card help

- [The Hitter Card / Baseball 365 help](https://365.strat-o-matic.com/help/hittercard)
- **Evidence:** publisher's digital-game help; relevant passages inspected.
- **Supports:** 50/50 card selection, weighted 2d6 rows, handedness columns, d20
  splits and N/W power's effect on pitcher-card HR chances.
- **Important boundary:** this page also describes electronic options such as
  "Correct Board Game Excesses." They are not evidence that the physical game
  implements the same corrections and cannot satisfy our no-device requirement.

### S03 — Reproduced Strat rules, including Advanced rule 12.1

- [CBA Strat-O-Matic rules reference](https://www.cba-bb.net/cba_stratomatic_rules_reference.htm)
- [Additional reproduced Super-Advanced instructions](https://baseballthinkfactory.org/btf/pages/basesim/somrules.htm)
- **Evidence:** unofficial hosting of rules; CBA power-rating passages read.
- **Supports:** N/W power and the specific pitcher-card HR-to-SINGLE** treatment
  for W hitters, including the split-result qualification.
- **Boundary:** not proof that this reproduction is the latest official edition.
  The second link was located as corroborating context, not independently audited
  in full. Do not extend one rule into a claim about all current formulas.

### S04 — Bruce Bundy, Doing the Splits, SOM-Style

- [Article and formulas](https://cba-bb.net/Bundy/Formit02.htm)
- **Evidence:** unofficial historical cardmaking analysis; article read.
- **Supports:** compensating for the average opposing card, weighted chances,
  d20 subchances and the precision-versus-speed tradeoff.
- **Boundary:** the article explicitly says only the game company has the correct
  formulas. It is a historical approximation; OCR artifacts and season-specific
  constants must not be treated as an authoritative current specification.

### S05 — Bruce Bundy, SOM Cardmaking Clinic

- [Formula summary](https://cba-bb.net/Bundy/Formit08.htm)
- **Evidence:** unofficial research; retrieved and relevant formula explanation
  examined through search results.
- **Supports:** replacing historical formula constants with the appropriate league
  mean and requiring L/R individual and league inputs for split cards.
- **Boundary:** no independent validation or modern proprietary-formula disclosure.

### S06 — mbutler/strat-o-rama: actual statistics-to-card code

- [Repository](https://github.com/mbutler/strat-o-rama)
- Inspected revision: **`08e5e53286d58f13255f5b814bca01f474fb4e91`**.
- Pinned files:
  - [README.md](https://github.com/mbutler/strat-o-rama/blob/08e5e53286d58f13255f5b814bca01f474fb4e91/README.md)
  - [player.js](https://github.com/mbutler/strat-o-rama/blob/08e5e53286d58f13255f5b814bca01f474fb4e91/player.js)
  - [card.js](https://github.com/mbutler/strat-o-rama/blob/08e5e53286d58f13255f5b814bca01f474fb4e91/card.js)
  - [notes.txt](https://github.com/mbutler/strat-o-rama/blob/08e5e53286d58f13255f5b814bca01f474fb4e91/notes.txt)
  - [config.js](https://github.com/mbutler/strat-o-rama/blob/08e5e53286d58f13255f5b814bca01f474fb4e91/config.js)
- **Evidence:** repository tree, README, notes and the three named JavaScript
  calculation/allocation/configuration files read. `app.js`'s PDF role is described
  in README/notes; its implementation was not inspected.
- **Supports:** Bundy-derived statistics-to-chance construction, subtraction of
  expected opposing-card contributions, weighted placement and d20 splits.
- **Limits found directly in source:** hardcoded baselines, negative-chance
  clipping, baseline constants labeled 2016 against configured 2018 input paths.
  These are limitations to investigate, not a completed runtime bug audit.
- **Explicit disclaimer:** unofficial, "plausible" cards; not affiliated with
  Strat-O-Matic and not compatible with the official game, per its README.
- **Not performed:** installation, dependency audit, generator execution,
  reconciliation of generated probabilities, full program review or comparison
  with official cards. Do not copy its constants or assume validation from its
  existence. Review code/data/artwork licenses separately before reuse.
- [Author's background article](https://mbutler.org/strat-o-magic/) was located
  through search; the pinned code, rather than that narrative, supports the
  implementation observations above.

### S07 — Printing and report tools: useful but different evidence

- [Netplay Baseball Create Cards Utility](https://netplaybaseball.com/utilities/create-baseball-cards.php)
- [Advanced text-input utility](https://netplaybaseball.com/utilities/create-baseball-cards-adv.php)
- [brencon/strat2json](https://github.com/brencon/strat2json)
- [mike14747/strat-ratings-guide](https://github.com/mike14747/strat-ratings-guide)
- **Evidence:** descriptions located through search, not source-code audits.
- **Use:** distinguish rendering existing card text, parsing reports and analyzing
  existing ratings from calculating new player probabilities. No claim that these
  tools reveal an official cardmaking algorithm or meet our card-size requirement.

### S08 — APBA official Master Game instructions

- [Official downloads index](https://www.apbagames.com/downloads)
- [Current linked manual path](https://www.apbagames.com/s/Baseball_Master_Game_Instructions.pdf)
- [Older indexed manual URL](https://apbagames.com/wp-content/uploads/baseball/Baseball_Master_Game_Instructions.pdf)
- **Evidence:** official downloads page read; relevant manual passages available
  in search indexing. They describe grades 1–30 and strikeout/control symbols.
- **Retrieval limitations:** the older URL returned HTTP 404; the current PDF
  request was aborted. A separately located mirror also failed to retrieve. A
  document-hosting page extracted unrelated recommended material and was not
  accepted as a reading of the manual. No complete current-manual inspection.
- **Boundary:** public mechanics are not disclosure of the current proprietary
  rating formulas. Do not treat Basic and Master editions as interchangeable.

### S09 — APBA worked explanations and unofficial grading analysis

- [Learning How To Play APBA Baseball](https://shlabotnikreport.wordpress.com/2020/07/28/learning-how-to-play-apba-baseball/)
- [Chris on APBA: On Pitching, part II](https://www.apbablog.com/card-analysis/chris-on-apba-on-pitching-part-ii)
- **Evidence:** Basic Game walkthrough read in relevant passages; grading article
  retrieved and its relevant descriptions surfaced through search.
- **Supports:** batter-card result numbers interpreted through base-state boards,
  pitcher grades and defensive ratings; more detailed pitcher symbols in Master
  play. The Basic walkthrough uses an older card set, not a current-edition audit.
- **Boundary:** unofficial walkthroughs and reverse-engineered grading discussion
  do not certify the publisher's complete current construction process.

### S10 — DYNASTY League's three-d10 precedent

- [Publisher's board-game explanation](https://pursuethepennant.com/dynasty-league-features.html)
- [Official 2024 rulebook located through search](https://www.dynastyleaguebaseball.com/Docs/DYNASTYRulebook2024v3.pdf)
- **Evidence:** publisher's HTML explanation read; PDF located/indexed, not read
  in full.
- **Supports:** three ordered decimal dice, 0–499 hitter and 500–999 pitcher,
  opponent-handedness columns and some printed advancement information.
- **Boundary:** precedent for physical play, not proof of quantitative accuracy,
  our proposed card legibility, or independent confirmation of marketing claims.

## Matchup modeling and sparse information

### S11 — SABR, Matchup Probabilities in Major League Baseball

- [Article and derivation](https://sabr.org/journal/article/matchup-probabilities-in-major-league-baseball/)
- **Evidence:** article read, including its sample and stated limitations.
- **Supports:** normalized multi-outcome extension using weights `B_k*P_k/L_k`;
  binary log5 as a special case.
- **Scope:** 2012 Retrosheet data, seven coarse categories, selected high-exposure
  hitters and pitchers (at least 502 opportunities). Its small head-to-head
  examples are explicitly illustrative, not conclusive.
- **Boundary:** aggregate agreement in that study is not proof of best predictive
  performance, sparse-player calibration, causal defense or full-game accuracy.

### S12 — Doo and Kim (2018), Bayesian matchup estimation

- [PLOS ONE article](https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0204874)
- **Evidence:** paper retrieved; methodology, target and evaluation passages read.
- **Supports:** hierarchical log5 estimation with prior information; reported
  improvement over its comparisons in a KBO example.
- **Boundary:** principally a binary on-base target, not a validated full MLB
  outcome/runner/defense simulator or universal best method.

### S13 — Powers, Hastie and Tibshirani, regularized multinomial outcomes

- [Paper abstract and publication details](https://arxiv.org/abs/1706.10272)
- **Evidence:** abstract/publication details read; no reproduction of the fits.
- **Supports:** regularized multinomial batter/pitcher outcome estimation that
  shares information across categories; a candidate comparison method.
- **Boundary:** a regularization method is not automatically Bayesian or superior
  on our data. Published in Statistical Modelling (2018), not a new 2026 result.

### S14 — O'Connell, linked matrix factorization for matchup batting averages

- [2024 preprint](https://arxiv.org/abs/2402.01914)
- [Inspected HTML version](https://arxiv.org/html/2402.01914v1)
- **Evidence:** abstract and introduction read, not an independent paper audit.
- **Supports:** more recent work incorporating linked batter/pitcher data; its
  stated target is batting average, and it reports a modest comparative gain.
- **Boundary:** newer does not mean better for all game outcomes, table size,
  interpretability or tabletop speed. No claim of best complete-game algorithm.

### S15 — Platoon-effect stabilization background

- [FanGraphs: Estimating Hitter Platoon Skill](https://blogs.fangraphs.com/estimating-hitter-platoon-skill/)
- **Evidence:** relevant regression explanation located through search; page
  retrieved. Its numerical prescriptions were not independently evaluated.
- **Supports:** player-specific platoon effects can require substantial regression
  toward appropriate group effects rather than literal small-sample splits.
- **Boundary:** do not copy historical fixed regression strengths without selecting
  and validating a model appropriate to our objectives and data.

## Data provenance and distribution constraints

### S16 — baseball.computer public data description

- [Homepage](https://baseball.computer/)
- **Evidence:** page read. It describes preview status, Retrosheet provenance,
  attribution and additional contributions under CC BY-SA 4.0.
- Required Retrosheet notice quoted there:

  > The information used here was obtained free of charge from and is copyrighted
  > by Retrosheet. Interested parties may contact Retrosheet at "www.retrosheet.org".

- **Boundary:** source availability is not proof that all seasons/fields are
  complete, that true range/arm can be identified, or that every bundled/derived
  product's rights are resolved.

### S17 — baseball.computer repository license

- [Repository LICENSE](https://github.com/droher/baseball.computer/blob/main/LICENSE)
- **Evidence:** raw license header read: Attribution-NonCommercial-ShareAlike 4.0
  International. This differs from the site's additional-data statement.
- **Boundary:** clarify which terms cover which materials before public/commercial
  distribution. No legal opinion, publisher outreach or distribution approval
  follows from this research or its local Git commit.
- Existing repository evidence:
  [internal go/no-go](../docs/data-audits/2019-go-no-go.md),
  [coverage inventory](../docs/data-audits/2019.md),
  [live progress and release hold](../docs/PROGRESS.md).

## Exploratory commentary, not accepted accuracy evidence

### S18 — Outside comparisons and outlier discussions

- [SABR Gaming blog: How to Easily Address Outliers in Strat-O-Matic Baseball](https://sabrbaseballgaming.com/2025/02/06/how-to-easily-fix-strat-o-matic-baseball/)
- [Cybermetrics: Log5 vs. Strat-O-Matic](https://cybermetric.blogspot.com/2023/03/log5-vs-strat-o-matic.html)
- [Sports Mogul: Strat-O-Matic Platoon Splits](https://files.sportsmogul.com/articles/strat-platoon-splits.pdf)
- **Evidence:** the outlier blog was read; the other two were located through
  search, not inspected as full supporting derivations.
- **Use:** leads on additive-model limitations and possible comparisons.
- **Not accepted:** numerical accuracy percentages, proof of a current-edition
  defect, or blanket claims that the actual game has no outlier safeguards. The
  documented N/W rule already illustrates why a criticism of a pure mixture
  cannot be applied unqualified to all of Strat. Our additive-limit examples are
  independently checked algebra, not reliance on an opinion piece's authority.

## What this research did not do

No external program was installed or executed. No full commercial game was
reverse-engineered, no current proprietary formula was obtained, and no source
season was downloaded for this archive. No quantitative comparison against actual
Strat/APBA/DYNASTY cards, new model fits, card-print tests or tabletop trials were
performed. External link reachability is not continuously monitored. The exact
arithmetic check in [DESIGN-AND-MATH.md](DESIGN-AND-MATH.md) is the reproducible
numerical evidence, not a certification of the sources or of game accuracy.
