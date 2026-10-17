# Card-based baseball game research

Research and recommendations recorded **2026-10-17** from the tabletop-game
discussion. This is an exploratory design archive, **not an implemented game,
a final rules specification, a model-selection result, or release approval**.
The repository's current work and next task remain in
[docs/PROGRESS.md](../docs/PROGRESS.md); this archive does not replace that ledger
or change the [product contract](../PLAN.md) or
[execution gates](../docs/EXECUTION.md).

## Confirmed requirements and original concept

- The entire finished game must be playable **offline without any computer,
  phone, electronic dice, app, network connection, or digital device**.
- Physical player cards, dice, printed rules/reference aids and pencil-and-paper
  scorekeeping must suffice. Software may manufacture cards and validate the
  design, but must not be required during play.
- Use ordinary playing-card size; poker size, **2.5 × 3.5 inches
  (63.5 × 88.9 mm)**, is the working layout assumption, not a completed print test.
- Batter and pitcher cards have separate opponent-handedness faces:
  **vs LHP / vs RHP** for batters and **vs LHB / vs RHB** for pitchers.
- Pair the current batter and pitcher and expose the appropriate faces.
- Physical dice may be of any number/type and kept in a fixed order. Three
  ordered d10s generating **000–999** are the leading candidate. A separate
  selector die is permitted, not required.
- The game should support meaningful batter/pitcher matchups, running, stealing,
  fielding range, errors and throwing, while remaining easy to learn and play
  without numerous complicated charts.
- Existing project statistics are a candidate foundation. Additional statistics
  from baseball.computer may be investigated if necessary; no new dataset
  extraction or publication is part of this archive.

Earlier discussion of digital dice or a matchup-calculating companion is
**superseded by the fully physical requirement**. A digital reference model is a
manufacturing/validation tool only, not an alternative mode required to play.

## Reading guide

| File | Contents |
| --- | --- |
| [DESIGN-AND-MATH.md](DESIGN-AND-MATH.md) | Card feasibility, matchup equations, dice resolution, candidate analog mechanics, defense/running, data limits and evaluation recommendations |
| [SOURCES.md](SOURCES.md) | Source links, primary versus unofficial evidence, pinned GitHub inspection, retrieval limitations and rights caveats |

## Main conclusions

1. **Card capacity looks feasible.** Compact number ranges encode many chances
   with few rows. An estimated 10–14 main result rows plus short rating strips is
   worth an actual-size prototype; legibility is not yet demonstrated.
2. **League-relative construction is necessary.** Raw batter/pitcher averaging
   regresses players toward average. In a simplified 50/50 model,
   `C_B = 2B − L` and `C_P = 2P − L` produce `B + P − L` overall. These are
   illustrative equations, not the complete proprietary Strat-O-Matic recipe.
3. **A pure additive mixture has limits.** Extreme rates can require negative
   card chances. Clipping them to zero changes the model; more dice do not fix
   that structural problem. Existing Strat rules already include cross-card
   exceptions such as N/W power.
4. **Three d10s are a credible starting point.** Splitting 000–499 onto the batter
   and 500–999 onto the pitcher gives 500 entries per selected card but 1,000
   equally likely primary matchup results: one slot contributes 0.10 percentage
   point overall. An independent 50/50 selector plus a full 000–999 table on each
   card halves that increment to 0.05 percentage point.
5. **Accuracy must be measured, not inferred from dice count or source quality.**
   Compare a league-adjusted baseline with coherent nonlinear reference models;
   measure statistical estimation error separately from physical-card
   approximation and rounding error. No superiority over Strat-O-Matic or APBA
   has been demonstrated.
6. **Resolve contested contact before final hit/out/error.** Defense must not be
   an independent reroll that turns an already finalized hit into an out. Running
   speed, steal attempts/success, range, errors and arms represent different
   abilities and require appropriate opportunity denominators.

## Recommended next design experiment — not implementation authorization

Start with a small, contrasting set of 2019 players rather than a complete season
or elaborate artwork. Compare a league-adjusted 50/50 baseline with a version
using only targeted opponent-interaction checks where the baseline fails.
Initially try three ordered d10s with 000–499 / 500–999 card selection, one small
shared reference aid, and rerolls only when needed. Add a selector or more primary
digits only if measured accuracy or usability justifies them. Compute exact
matchup probabilities, then print at actual size and test playing speed.

Open choices include the final interaction rules, rating scales, treatment of
rare events, reference-aid size, endurance/parks/era rules, quantitative accuracy
bounds, and the balance between reproducing a selected season and estimating
underlying ability. No choice here authorizes new fitting or waives project gates.

## Reproduce the documented checks

Run from the repository root. These checks need only Python's standard library
and Git; they do not read source datasets, call the network, or run the game.

### Exact arithmetic and dice-space checks

The Python block under `## Reproducible arithmetic` in
[DESIGN-AND-MATH.md](DESIGN-AND-MATH.md) is executable evidence, not a simulation.
Extract and run it without installing anything:

```sh
python3 - <<'PY'
from pathlib import Path
text = Path('CARD-BASED-GAME-RESEARCH/DESIGN-AND-MATH.md').read_text()
section = text.split('## Reproducible arithmetic\n', 1)[1]
code = section.split('```python\n', 1)[1].split('\n```', 1)[0]
exec(compile(code, 'documented-card-math', 'exec'))
PY
```

### Research-package local links and whitespace

This checks path targets for inline relative Markdown links in the three research
documents. It does not validate external URLs or heading anchors.

```sh
python3 - <<'PY'
from pathlib import Path
import re
from urllib.parse import unquote, urlsplit
root = Path('CARD-BASED-GAME-RESEARCH')
files = sorted(root.glob('*.md'))
assert len(files) == 3, [str(p) for p in files]
count = 0
for path in files:
    for target in re.findall(r'(?<!!)\[[^\]\n]+\]\(([^)\s]+)\)', path.read_text()):
        url = urlsplit(target)
        if url.scheme or url.netloc or not url.path:
            continue
        assert (path.parent / unquote(url.path)).exists(), (str(path), target)
        count += 1
print(f'PASS: {count} local Markdown link targets across {len(files)} documents')
PY
git diff --check -- CARD-BASED-GAME-RESEARCH docs/PROGRESS.md
git diff --cached --check -- CARD-BASED-GAME-RESEARCH docs/PROGRESS.md
```

Documentation and arithmetic checks do **not** establish game accuracy. No card
printing/readability test, tabletop playtest, new model fitting, full-season replay,
external generator execution or commercial-game benchmark has been performed for
this research. No production code changes are part of the package; engine/model
suite results belong to their own separately verified work packages.
