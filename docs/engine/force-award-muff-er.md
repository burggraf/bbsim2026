# E-03 — exact protected force awards after an empty-base muff

## Preregistration and canonical flow

Selected after verified local `8f059899665548dec9220b1d2f08fa7427b9bd0d`,
before new tests/production. Inspected `protected_award::validate`, the exact
force-chain/clean predicate, current live-error versus permanent certificate
markers and existing station/terminal helpers. Primary 2019 Rules5.05(b)(1)–(2),
5.06(b)(3)(B), 9.16 opening/(a), (b)(3), (f), (i); rulebook SHA256
`d85e0e43cffaff4ed916065d5434b26b0d93d54dbd91630d585eeefdfca29e92`.

## Bounded extension and reconstruction argument

Allow canonical ProtectedAwardFacts under whole-chance NoMisplay review only
when the existing exact forced-award predicate also holds. Keep the observed
empty-base single missed-before-first retirement at0/1outs, clean zero-run
prefix, same pitcher/no personnel, qualified suffix and legal completion.
Ordinary/intentional BB and HBP supply only batter First and the uninterrupted
forced chain; unforced runners hold, explicitly or by omission.

The original error reach is the leading runner until it scores: it starts
alone, station hits preserve runner order, held retirements cannot remove it,
and exact force awards only advance their uninterrupted chain. Deleting this
leading error runner never changes a following clean runner's forced advance:
a First runner is still forced by the awarded batter; the consecutive Second
runner is still forced by that First runner. No clean runner lies beyond the
removed leading runner. Gaps stop the same clean force chain in either history.
Once the error reach scores, all actual remaining runners have clean reaches
and identical virtual positions. This is not a blanket independence claim for
unforced advancement, FC, extra misplays or arbitrary continuous hits.

The live original-error marker excludes only that reach; clean award/hit/HR
runners scoring before two actual outs can earn. Retain virtual-third-out
arithmetic, marker reset and prior unknown irreversibility. Missing cause,
review, origin/completion, occupied muff bases, personnel/relief, pre-scoring
and unsupported advancement remain unknown. No DTO/codec/physical/RBI/producer,
RNG, probabilities or public API change; clean/two-out certificates unchanged.

## Observable acceptance before implementation

-576 literal frames: both halves × initial0/1outs ×1–4 awards × ordinary BB,
 IBB,HBP × awards-HR/K-awards-HR/awards-K-HR × independent move/credit reversal.
 Actual R=n+2, BF=n+5, AB5/H1/TB4/outs3/K3 and named E1/catcher PO3/A0/PB0.
 ER=n+1 unless initial1out plus K before awards gives0, or initial1out plus K
 after awards gives max(n−3,0): fourth award already scored a clean runner.
 Assert direct constructor/codec/continuity and literal inning/reset credits.
- Leading error runner on Third and clean runner Second after a bound station
 double: exact award leaves both unforced; clearing HR gives R4/ER3 and AB6,
 BF7/H2/TB6/K3/outs3. Both halves/initialouts/vector orders; explicit/omitted
 unforced holds must both qualify without fabricating movement chronology.
- Nine consecutive awards plus HR: R11/ER10/BF14/H1/AB5/K3/outs3/TB4; original
 error actor returns with a distinct clean award and later earns. Test kinds
 and vector order. Ordinary loaded-award error-run walkoff R1/ER0 uses actual
 legal completion, not a fabricated physical third out.
- Missing award cause/review keeps identical physical deltas but ER unknown;
 excess/unforced advance, earlier unknown, relief and personnel retain existing
 conservative guards. Preserve all historical accounting assertions byte-exact;
 no oracle migration. Freeze production and witness genuine semantic reds,
 separating fixture/compiler failures, before implementation.
- Full applicable checks/raw logs, complete actual staged inspection and exact
 working/index/frozen evidence before local checkpoint. No push, independent
 review, full-box or E-03/G1 completion claim.

[PROGRESS.md](../PROGRESS.md) is the only live verification/status ledger.
