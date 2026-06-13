# Ternary Anchor

**Ternary Anchor** provides stability and persistence primitives for rooms in dynamic ternary fleet environments — modeling how agents hold position against drift using anchor states (Stowed, Deployed, Set, Dragging).

## Why It Matters

In a dynamic fleet, agents must sometimes hold a fixed position (monitoring a sensor, guarding a resource) rather than exploring. Without explicit anchoring, drift accumulates from environmental forces (network jitter, load rebalancing, leader changes). Ternary Anchor provides a maritime-inspired anchor lifecycle: deploy (drop anchor), set (dig in), detect drag (slipping), and stow (release). This mirrors real-world distributed systems where services must maintain sticky sessions, pinned connections, or quorum membership.

## How It Works

### Anchor State Machine

```
Stowed ──deploy(pos)──→ Deployed ──set()──→ Set
                           ↑                    │
                           │                  drag()
                           │                    ↓
                           └────────────── Dragging
                           
Set (holding) ──stow()──→ Stowed
Dragging ──stow()──→ Stowed
```

State transitions: **O(1)**. Each transition enforces validity — you can't `set()` from `Stowed`.

### Hold Strength

Anchors have `hold_strength: u8` (0-100), representing grip quality:

```
deploy: strength = 100
wear(amount): strength = saturating_sub(strength, amount)
```

When strength drops below threshold, the anchor is likely to drag. Strength tracking: **O(1)**.

### Drift Detection

The `DriftMonitor` tracks position deviation:

```
expected_position vs actual_position
drift = ||actual - expected||
if drift > threshold: trigger alert
```

Drift computation: **O(1)** (two coordinate subtractions). Configurable threshold per anchor.

### Chains and Anchorage

- `Chain`: Sequence of linked anchors providing redundant holding
- `Anchorage`: Designated safe zone with pre-validated holding positions
- `WeighAnchor`: Command to release all anchors and resume navigation

Chain evaluation: **O(N)** for N linked anchors. Anchorage lookup: **O(1)** via HashMap.

## Quick Start

```rust
use ternary_anchor::Anchor;

let mut anchor = Anchor::new();
anchor.deploy((100, 200));
anchor.set();

assert!(anchor.is_holding());
println!("Strength: {}", anchor.hold_strength()); // 100

anchor.wear(30);
println!("Strength: {}", anchor.hold_strength()); // 70
```

## API

| Type | Description |
|------|-------------|
| `Anchor` | Position holder with state, hold_strength, wear tracking |
| `AnchorState` | `Stowed`, `Deployed`, `Set`, `Dragging` |
| `DriftMonitor` | Position deviation detector |
| `Chain` | Linked anchor sequence for redundancy |

Key methods: `deploy(pos)`, `set()`, `drag()`, `stow()`, `wear(amount)`, `is_holding()`.

## Architecture Notes

Ternary Anchor provides fleet stability primitives in SuperInstance. In γ + η = C, anchoring is η (avoidance — preventing drift from strategic positions) while weighing anchor resumes γ (growth — exploration of new positions). Integrates with `ternary-beacon` for anchored agent discovery and `ternary-compass` for drift orientation.

See [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md) for fleet stability architecture.

## References

1. Lynch, N. (1996). *Distributed Algorithms*. Morgan Kaufmann. Chapter 17: Failure Detectors.
2. Fischer, M. et al. (1985). "Impossibility of Distributed Consensus with One Faulty Process." *JACM*, 32(2).
3. Gilbert, S. & Lynch, N. (2002). "Brewer's Conjecture and the Feasibility of Consistent, Available, Partition-Tolerant Web Services." *ACM SIGACT News*.

## License

MIT
