#![forbid(unsafe_code)]

//! ternary-anchor: Stability and persistence for rooms in dynamic fleet environments.
//!
//! Provides anchor-level abstractions for holding position: anchors, chains,
//! drift monitoring, weighing anchor, finding stable ground, and designated
//! anchorage zones. Maps to how rooms maintain position when not navigating.

use std::time::Instant;

/// The state of an anchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchorState {
    Stowed,
    Deployed,
    Set,
    Dragging,
}

/// An anchor that holds a room in position.
#[derive(Debug, Clone)]
pub struct Anchor {
    state: AnchorState,
    position: (i64, i64),
    hold_strength: u8,
}

impl Anchor {
    pub fn new() -> Self {
        Anchor {
            state: AnchorState::Stowed,
            position: (0, 0),
            hold_strength: 100,
        }
    }

    /// Deploy the anchor at a given position.
    pub fn deploy(&mut self, pos: (i64, i64)) {
        self.position = pos;
        self.state = AnchorState::Deployed;
    }

    /// Anchor has set (dug in and holding).
    pub fn set(&mut self) {
        if self.state == AnchorState::Deployed {
            self.state = AnchorState::Set;
        }
    }

    /// Anchor is dragging — no longer holding.
    pub fn drag(&mut self) {
        self.state = AnchorState::Dragging;
    }

    /// Stow the anchor.
    pub fn stow(&mut self) {
        self.state = AnchorState::Stowed;
    }

    pub fn state(&self) -> AnchorState {
        self.state
    }

    pub fn position(&self) -> (i64, i64) {
        self.position
    }

    pub fn is_holding(&self) -> bool {
        self.state == AnchorState::Set
    }

    pub fn hold_strength(&self) -> u8 {
        self.hold_strength
    }

    /// Reduce hold strength (simulates wear).
    pub fn wear(&mut self, amount: u8) {
        self.hold_strength = self.hold_strength.saturating_sub(amount);
    }
}

impl Default for Anchor {
    fn default() -> Self {
        Self::new()
    }
}

/// Connection from the room to stable ground.
#[derive(Debug, Clone)]
pub struct AnchorChain {
    length: u32,
    deployed_length: u32,
    scope: f64,
}

impl AnchorChain {
    pub fn new(length: u32) -> Self {
        AnchorChain {
            length,
            deployed_length: 0,
            scope: 1.0,
        }
    }

    /// Pay out chain to a given depth.
    pub fn pay_out(&mut self, amount: u32) {
        self.deployed_length = (self.deployed_length + amount).min(self.length);
    }

    /// Retrieve chain.
    pub fn retrieve(&mut self, amount: u32) {
        self.deployed_length = self.deployed_length.saturating_sub(amount);
    }

    /// Retrieve all chain.
    pub fn retrieve_all(&mut self) {
        self.deployed_length = 0;
    }

    /// Set the scope ratio (chain length / depth).
    pub fn set_scope(&mut self, scope: f64) {
        self.scope = scope.max(1.0);
    }

    pub fn deployed_length(&self) -> u32 {
        self.deployed_length
    }

    pub fn total_length(&self) -> u32 {
        self.length
    }

    pub fn scope(&self) -> f64 {
        self.scope
    }

    pub fn remaining(&self) -> u32 {
        self.length - self.deployed_length
    }
}

/// Monitor for anchor drift.
#[derive(Debug, Clone)]
pub struct AnchorWatch {
    origin: (i64, i64),
    tolerance: u64,
    alerted: bool,
}

impl AnchorWatch {
    pub fn new(origin: (i64, i64), tolerance: u64) -> Self {
        AnchorWatch {
            origin,
            tolerance,
            alerted: false,
        }
    }

    /// Check if current position indicates drift.
    pub fn check(&mut self, current: (i64, i64)) -> bool {
        let dx = (current.0 - self.origin.0).unsigned_abs();
        let dy = (current.1 - self.origin.1).unsigned_abs();
        let drifting = dx > self.tolerance || dy > self.tolerance;
        if drifting {
            self.alerted = true;
        }
        drifting
    }

    pub fn is_alerted(&self) -> bool {
        self.alerted
    }

    /// Acknowledge and clear the alert.
    pub fn acknowledge(&mut self) {
        self.alerted = false;
    }
}

/// Command to weigh (retrieve) the anchor and prepare for movement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeighAnchor {
    reason: String,
    issued_at: Instant,
}

impl WeighAnchor {
    pub fn new(reason: impl Into<String>) -> Self {
        WeighAnchor {
            reason: reason.into(),
            issued_at: Instant::now(),
        }
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// Find a stable position in the fleet topology.
#[derive(Debug, Clone)]
pub struct AnchorGround {
    positions: Vec<(i64, i64, u8)>, // (x, y, stability 0-100)
    best: Option<(i64, i64)>,
}

impl AnchorGround {
    pub fn new() -> Self {
        AnchorGround {
            positions: Vec::new(),
            best: None,
        }
    }

    /// Add a candidate position with its stability score.
    pub fn add_candidate(&mut self, x: i64, y: i64, stability: u8) {
        self.positions.push((x, y, stability));
        self.best = None; // invalidate cache
    }

    /// Get the most stable position.
    pub fn find_best(&mut self) -> Option<(i64, i64)> {
        if self.best.is_some() {
            return self.best;
        }
        let best = self
            .positions
            .iter()
            .max_by_key(|p| p.2)
            .map(|p| (p.0, p.1));
        self.best = best;
        best
    }

    pub fn candidate_count(&self) -> usize {
        self.positions.len()
    }

    /// Clear all candidates.
    pub fn clear(&mut self) {
        self.positions.clear();
        self.best = None;
    }
}

impl Default for AnchorGround {
    fn default() -> Self {
        Self::new()
    }
}

/// A designated stable zone where rooms can anchor.
#[derive(Debug, Clone)]
pub struct Anchorage {
    name: String,
    center: (i64, i64),
    radius: u64,
    capacity: usize,
    occupied: usize,
}

impl Anchorage {
    pub fn new(name: impl Into<String>, center: (i64, i64), radius: u64, capacity: usize) -> Self {
        Anchorage {
            name: name.into(),
            center,
            radius,
            capacity,
            occupied: 0,
        }
    }

    /// Check if a position is within this anchorage.
    pub fn contains(&self, pos: (i64, i64)) -> bool {
        let dx = (pos.0 - self.center.0).unsigned_abs();
        let dy = (pos.1 - self.center.1).unsigned_abs();
        dx <= self.radius && dy <= self.radius
    }

    /// Try to moor. Returns false if full.
    pub fn moor(&mut self) -> bool {
        if self.occupied < self.capacity {
            self.occupied += 1;
            true
        } else {
            false
        }
    }

    /// Unmoor a room.
    pub fn unmoor(&mut self) {
        self.occupied = self.occupied.saturating_sub(1);
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn center(&self) -> (i64, i64) {
        self.center
    }

    pub fn radius(&self) -> u64 {
        self.radius
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn occupied(&self) -> usize {
        self.occupied
    }

    pub fn available(&self) -> usize {
        self.capacity - self.occupied
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_new_stowed() {
        let a = Anchor::new();
        assert_eq!(a.state(), AnchorState::Stowed);
    }

    #[test]
    fn anchor_deploy_and_set() {
        let mut a = Anchor::new();
        a.deploy((10, 20));
        assert_eq!(a.state(), AnchorState::Deployed);
        assert_eq!(a.position(), (10, 20));
        a.set();
        assert_eq!(a.state(), AnchorState::Set);
        assert!(a.is_holding());
    }

    #[test]
    fn anchor_drag() {
        let mut a = Anchor::new();
        a.deploy((0, 0));
        a.set();
        a.drag();
        assert_eq!(a.state(), AnchorState::Dragging);
        assert!(!a.is_holding());
    }

    #[test]
    fn anchor_stow() {
        let mut a = Anchor::new();
        a.deploy((0, 0));
        a.stow();
        assert_eq!(a.state(), AnchorState::Stowed);
    }

    #[test]
    fn anchor_wear() {
        let mut a = Anchor::new();
        assert_eq!(a.hold_strength(), 100);
        a.wear(30);
        assert_eq!(a.hold_strength(), 70);
        a.wear(200);
        assert_eq!(a.hold_strength(), 0);
    }

    #[test]
    fn chain_pay_out_and_retrieve() {
        let mut c = AnchorChain::new(100);
        c.pay_out(40);
        assert_eq!(c.deployed_length(), 40);
        assert_eq!(c.remaining(), 60);
        c.retrieve(10);
        assert_eq!(c.deployed_length(), 30);
    }

    #[test]
    fn chain_pay_out_capped() {
        let mut c = AnchorChain::new(50);
        c.pay_out(200);
        assert_eq!(c.deployed_length(), 50);
    }

    #[test]
    fn chain_retrieve_all() {
        let mut c = AnchorChain::new(100);
        c.pay_out(60);
        c.retrieve_all();
        assert_eq!(c.deployed_length(), 0);
    }

    #[test]
    fn chain_scope() {
        let mut c = AnchorChain::new(100);
        c.set_scope(5.0);
        assert_eq!(c.scope(), 5.0);
        c.set_scope(0.5);
        assert_eq!(c.scope(), 1.0); // min 1.0
    }

    #[test]
    fn watch_no_drift() {
        let mut w = AnchorWatch::new((0, 0), 10);
        assert!(!w.check((5, 5)));
        assert!(!w.is_alerted());
    }

    #[test]
    fn watch_detects_drift() {
        let mut w = AnchorWatch::new((0, 0), 10);
        assert!(w.check((20, 5)));
        assert!(w.is_alerted());
    }

    #[test]
    fn watch_acknowledge() {
        let mut w = AnchorWatch::new((0, 0), 5);
        w.check((100, 0));
        assert!(w.is_alerted());
        w.acknowledge();
        assert!(!w.is_alerted());
    }

    #[test]
    fn weigh_anchor_creation() {
        let wa = WeighAnchor::new("repositioning");
        assert_eq!(wa.reason(), "repositioning");
    }

    #[test]
    fn anchor_ground_find_best() {
        let mut ag = AnchorGround::new();
        ag.add_candidate(0, 0, 50);
        ag.add_candidate(10, 10, 90);
        ag.add_candidate(5, 5, 30);
        assert_eq!(ag.find_best(), Some((10, 10)));
        assert_eq!(ag.candidate_count(), 3);
    }

    #[test]
    fn anchor_ground_empty() {
        let mut ag = AnchorGround::new();
        assert_eq!(ag.find_best(), None);
    }

    #[test]
    fn anchor_ground_clear() {
        let mut ag = AnchorGround::new();
        ag.add_candidate(0, 0, 50);
        ag.clear();
        assert_eq!(ag.candidate_count(), 0);
    }

    #[test]
    fn anchorage_contains() {
        let a = Anchorage::new("home", (0, 0), 10, 5);
        assert!(a.contains((5, 5)));
        assert!(!a.contains((15, 0)));
    }

    #[test]
    fn anchorage_moor_and_unmoor() {
        let mut a = Anchorage::new("dock", (0, 0), 10, 2);
        assert!(a.moor());
        assert_eq!(a.occupied(), 1);
        assert!(a.moor());
        assert_eq!(a.occupied(), 2);
        assert!(!a.moor()); // full
        a.unmoor();
        assert_eq!(a.available(), 1);
    }

    #[test]
    fn anchorage_properties() {
        let a = Anchorage::new("bay", (10, 20), 50, 10);
        assert_eq!(a.name(), "bay");
        assert_eq!(a.center(), (10, 20));
        assert_eq!(a.radius(), 50);
        assert_eq!(a.capacity(), 10);
    }

    #[test]
    fn anchor_set_without_deploy() {
        let mut a = Anchor::new();
        a.set(); // should NOT transition from Stowed to Set
        assert_eq!(a.state(), AnchorState::Stowed);
    }
}
