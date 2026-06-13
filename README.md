# ternary-anchor

**Stability and persistence for rooms in dynamic ternary fleet environments.**

`ternary-anchor` provides anchor-level abstractions for holding position in a distributed fleet topology. It models anchors, chains, drift monitoring, and designated anchorage zones — the physical metaphors mapped to how rooms maintain positional stability when not actively navigating.

## Why It Matters

In a dynamic fleet of cooperating agents, rooms (logical spaces) must sometimes *stop moving* and hold a fixed position — to synchronize, process data, or wait for downstream capacity. Without an explicit anchoring abstraction, position drift propagates silently through the system, causing topology desynchronization and race conditions.

This crate formalizes the lifecycle: **deploy → set → drag → stow**, with quantitative drift detection and capacity-bounded anchorage zones. The anchor watch mechanism provides O(1) drift checks per tick, enabling real-time stability monitoring across thousands of rooms.

## How It Works

### Anchor State Machine

The anchor follows a constrained finite state machine (FSM) with four states and guarded transitions:

```
Stowed ──deploy()──▶ Deployed ──set()──▶ Set ──drag()──▶ Dragging
   ▲                    │                    │               │
   └──────stow()────────┴──────stow()────────┴───────────────┘
```

State transitions are guarded: `set()` only succeeds from `Deployed`, preventing invalid shortcuts. The `hold_strength` field (0–100) models mechanical wear via saturating subtraction:

$$h' = \max(0,\; h - \Delta w)$$

where $h \in [0, 100]$ is the current hold strength and $\Delta w$ is the wear amount. The saturating subtraction ensures $h' \geq 0$ without panicking on overflow.

**Time complexity:** All transitions are $O(1)$ — constant-time enum assignments with no allocation.

### Anchor Chain and Scope

The chain model computes the **scope ratio** — the ratio of deployed chain length to depth. In maritime engineering, a scope of 5:1–7:1 is standard for holding. This crate enforces a minimum scope of 1.0:

$$\text{scope} = \frac{L_{\text{deployed}}}{d} \geq 1.0$$

Chain operations use saturating arithmetic to prevent underflow on retrieval beyond deployed length:

$$L'_{\text{deployed}} = \max(0,\; L_{\text{deployed}} - \Delta L)$$

**Complexity:** $O(1)$ for all chain operations (`pay_out`, `retrieve`, `retrieve_all`).

### Drift Detection (Anchor Watch)

Given an origin position $(x_0, y_0)$ and tolerance $\tau$, the watch checks whether the current position has exceeded the **Chebyshev distance** threshold:

$$d_\infty = \max(|x - x_0|,\; |y - y_0|) > \tau$$

Using Chebyshev distance ($L_\infty$ norm) rather than Euclidean distance makes the check $O(1)$ with **no floating-point operations** — only integer subtraction, absolute value, and comparison. This is critical for real-time fleet monitoring where thousands of rooms must be checked per tick.

The Chebyshev metric defines a square boundary rather than a circle, which is appropriate for grid-based fleet topologies where movement is axis-aligned.

### Stable Ground Selection

`AnchorGround` finds the most stable position from a set of candidates. Each candidate carries a stability score $s \in [0, 100]$. Selection is:

$$\text{best} = \arg\max_{(x,y) \in C} s(x,y)$$

**Complexity:** $O(n)$ for $n$ candidates during the initial scan, with $O(1)$ cached lookups via memoization after the first computation. The cache is invalidated when new candidates are added.

### Anchorage Zones

An anchorage is a bounded region with capacity constraints. Containment uses the same Chebyshev metric:

$$\text{contains}(p) \iff |p_x - c_x| \leq r \;\wedge\; |p_y - c_y| \leq r$$

Mooring is a capacity-bounded increment with a boolean guard:

$$\text{moor}() = \begin{cases} \text{true}, & \text{occupied} < \text{capacity} \\ \text{false}, & \text{otherwise} \end{cases}$$

**Complexity:** $O(1)$ for containment check, moor, and unmoor operations.

## Quick Start

```toml
[dependencies]
ternary-anchor = "0.1"
```

```rust
use ternary_anchor::{Anchor, AnchorState, AnchorWatch, Anchorage, AnchorChain};

// Deploy and set an anchor
let mut anchor = Anchor::new();
anchor.deploy((10, 20));
anchor.set();
assert!(anchor.is_holding());
assert_eq!(anchor.hold_strength(), 100);

// Model wear over time
anchor.wear(30);
assert_eq!(anchor.hold_strength(), 70);

// Monitor for drift using Chebyshev distance
let mut watch = AnchorWatch::new((10, 20), 5);
assert!(!watch.check((12, 22)));  // d∞ = max(2,2) = 2 ≤ 5
assert!(watch.check((20, 5)));    // d∞ = max(10,15) = 15 > 5

// Manage chain scope
let mut chain = AnchorChain::new(100);
chain.pay_out(40);
chain.set_scope(5.0);
assert_eq!(chain.remaining(), 60);

// Use an anchorage zone with capacity
let mut zone = Anchorage::new("home-base", (0, 0), 50, 10);
assert!(zone.moor());
assert_eq!(zone.available(), 9);
```

## API

| Type | Purpose | Key Methods |
|------|---------|-------------|
| `Anchor` | Core anchor FSM with hold strength | `deploy()`, `set()`, `drag()`, `stow()`, `wear()` |
| `AnchorChain` | Chain with scope ratio management | `pay_out()`, `retrieve()`, `retrieve_all()`, `set_scope()` |
| `AnchorWatch` | O(1) Chebyshev drift detector | `check()`, `is_alerted()`, `acknowledge()` |
| `WeighAnchor` | Command object for anchor retrieval | `reason()`, `issued_at()` |
| `AnchorGround` | Candidate position selector with caching | `add_candidate()`, `find_best()`, `clear()` |
| `Anchorage` | Capacity-bounded mooring zone | `contains()`, `moor()`, `unmoor()`, `available()` |

## Architecture Notes

The anchor system maps to the SuperInstance conservation law **γ + η = C** (growth plus entropy equals constant). An anchor in the **Set** state represents low entropy ($\eta \to 0$) — energy is invested in holding position, and the system is maximally ordered. When the anchor drags, entropy increases as positional information degrades. The `wear()` operation models the gradual conversion of growth capacity ($\gamma$) into entropy as hold strength diminishes:

$$\gamma_{\text{available}} = C - \eta_{\text{wear}}$$

The anchorage capacity bound ensures the total "holding energy" of the fleet remains bounded by $C$, preventing resource exhaustion. When anchorage is full (`occupied = capacity`), no further growth is possible in that zone — a direct enforcement of the conservation law.

## References

- Cook, S.A. *The Complexity of Theorem-Proving Procedures.* STOC 1971. — FSM state complexity bounds.
- Tanenbaum, A.S. & Van Steen, M. *Distributed Systems: Principles and Paradigms.* Ch. 6, on coordination and consensus.
- Lloyd's Register. *Rules and Regulations for the Classification of Ships.* — Anchor scope and holding power standards.
- Cover, T.M. & Thomas, J.A. *Elements of Information Theory.* — Entropy and information-theoretic bounds.

## License

MIT
