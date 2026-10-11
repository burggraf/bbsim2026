# Run the scripted engine demo

This is a **visible engine proof**, not a probability model, historical replay,
complete official box or E-03/G1 signoff. The teams and players are fictional.
No data download, extra dependency or desktop setup is needed; use installed Rust.

From the repository root:

```sh
cargo run --manifest-path engine/Cargo.toml --example scripted_game
```

Press **Enter** for the next play, or **q** to stop. End-of-input also stops
cleanly. Rerun the command to restart the same game. For the whole game at once:

```sh
cargo run --manifest-path engine/Cargo.toml --example scripted_game -- --autoplay
```

## What you should see

Away opens with a single, stolen base, walk and three-run homer. Home answers
with a solo homer. The remaining outs are scripted strikeouts; both teams play
nine innings and Away wins **3–1**. Each of the **59 events** displays the play,
score, outs and named base occupants. An inning-ending play shows the engine's
normalized next-half state; the terminal play explicitly shows GAME OVER instead
of an active zero-out half. The final screen shows inning-by-inning runs and
supplied-credit batting/pitching statistics. Unknown AB would display `?`, not
zero. Omitted PO/A/RBI are explicitly not presented as complete official cells.

The example constructs explicit events, calls the real `apply` reducer, and
uses `derive_accounting` for final statistics. The presentation does not mutate
the score, bases or statistics itself. No random outcomes, real-player ratings,
manager policy or historical accuracy are demonstrated. The runner-only steal
does not consume a batting-order slot or PA.

## Verification and next product checkpoint

[Regression tests](../../engine/tests/scripted_demo.rs) require deterministic
59-event termination, continuity, codec round trips, runs3–1, PA30–28, hits2–1
and27 outs per pitcher, displayed play/base/score/stat labels, step/autoplay
completion, terminal GAME OVER presentation and clean EOF/quit. [The example](../../engine/examples/scripted_game.rs)
uses the [shared script/presentation](../../engine/examples/support/demo.rs).
Exact fresh commands/results belong to [PROGRESS](../PROGRESS.md).

This is a deliberately small developer preview. The next main-path planning
step is a gap check against the approved E-03 accounting matrix and literal
full-game evidence, then a bounded route to P-01 probabilities and P-02 generated
games. Do not expand rare recovery/appeal rules by default, add a desktop scaffold
before a working generated game, or count this scripted result as model accuracy.
