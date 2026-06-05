# ternary-anchor: Stability and persistence in dynamic fleet environments

`Anchor`, `AnchorChain`, `AnchorWatch`, `WeighAnchor`, `AnchorGround`, and `Anchorage` — everything a room needs to hold position when it's not actively navigating.

## Why This Exists

Rooms in a fleet topology don't always need to move. Sometimes the right thing is to stay put — hold a strategic position, wait for conditions to change, or provide a stable reference point for other rooms. This crate maps nautical anchoring onto fleet stability: deploying anchors, paying out chain, monitoring for drift, and finding good places to hold.

## Core Concepts

- **Anchor** — Holds a room at a position. Transitions through states: `Stowed` → `Deployed` → `Set` → (optionally `Dragging`).
- **AnchorChain** — Connection from the room to the anchor. Has a fixed length and tracks how much chain is paid out. The *scope* (chain-to-depth ratio) determines holding power.
- **AnchorWatch** — Monitors for drift by comparing current position against the origin. Fires an alert when the room moves beyond tolerance.
- **WeighAnchor** — A command to retrieve the anchor and prepare for movement. Carries a reason and timestamp.
- **AnchorGround** — Finds the most stable position among candidates. Each candidate has a stability score.
- **Anchorage** — A designated stable zone with a center, radius, and capacity. Rooms moor and unmoor as they arrive and leave.

## Quick Start

```toml
[dependencies]
ternary-anchor = "0.1"
```

```rust
use ternary_anchor::{Anchor, AnchorWatch, Anchorage};

// Deploy an anchor
let mut anchor = Anchor::new();
anchor.deploy((10, 20));
anchor.set();
assert!(anchor.is_holding());

// Watch for drift
let mut watch = AnchorWatch::new((10, 20), 5);
assert!(!watch.check((12, 22))); // within tolerance
assert!(watch.check((20, 25)));  // drifted!

// Use an anchorage zone
let mut dock = Anchorage::new("home-base", (0, 0), 50, 3);
assert!(dock.moor());
assert_eq!(dock.available(), 2);
```

## API Overview

| Type | What it is |
|------|-----------|
| `Anchor` | Holds a room at a fixed position with state tracking |
| `AnchorChain` | Connection with controllable length and scope |
| `AnchorWatch` | Drift monitor that alerts when position changes |
| `WeighAnchor` | Command to retrieve anchor and prepare for movement |
| `AnchorGround` | Finds the most stable position among candidates |
| `Anchorage` | Designated stable zone with capacity limits |
| `AnchorState` | Enum: Stowed, Deployed, Set, Dragging |

## How It Works

An `Anchor` is a state machine. It starts `Stowed`, transitions to `Deployed` when dropped at a position, then `Set` when it digs in. If the room drifts, it enters `Dragging`. The anchor tracks wear — repeated stress reduces `hold_strength` from 100 toward 0.

`AnchorWatch` is a simple distance check: it records the origin position and a tolerance. Each call to `check` compares the Manhattan distance in each axis against tolerance. It's intentionally not Euclidean — ternary topology moves in discrete steps.

`Anchorage` manages a fixed-capacity zone. Rooms call `moor` to claim a slot and `unmoor` to release. When full, `moor` returns false. The `contains` method checks whether a position falls within the anchorage's bounding box.

## Known Limitations

- `AnchorWatch` uses Manhattan distance (per-axis), not Euclidean. This matches ternary topology but may be surprising for continuous coordinates.
- `AnchorGround` does not update stability scores dynamically — you must clear and re-add candidates when conditions change.
- `Anchorage` uses a square bounding box, not a circle. `contains((x, y))` checks `|dx| <= radius && |dy| <= radius`.
- No persistence: `WeighAnchor` uses `Instant` which is not serializable.

## Use Cases

- **Holding pattern**: A room completes its objective and anchors at its current position, waiting for new orders.
- **Drift detection**: An `AnchorWatch` alerts when a room unexpectedly moves — useful for detecting topology instability.
- **Staging zones**: An `Anchorage` with capacity limits ensures not too many rooms cluster in one area.

## Ecosystem Context

Part of the SuperInstance ternary fleet library. This is the *stability* layer — the counterpart to `ternary-helm` (movement). A room typically alternates: navigate with helm, then anchor in place. `ternary-anchor` also supports `ternary-current` by providing stable reference points for information flow.

## License

MIT
