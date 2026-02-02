//! FiberBundle — a functional grouping of fiber tracts.
//!
//! A bundle represents one functional pathway (e.g., "left arm", "vocal tract",
//! "gaze control"). It contains multiple tracts of different kinds — motor for
//! commands, sensory for feedback. The bundle is the unit of embodiment.
//!
//! Chemical modulation affects entire bundles. Adrenaline dumps increase gain
//! on all motor tracts. Endorphins reduce sensitivity on nociceptive tracts.
//! This is how the body works: chemistry is broadcast, not targeted.

use crate::tract::{FiberTract, FiberTractKind};

/// A named bundle of fiber tracts forming a functional pathway.
///
/// Bundles group the tracts that serve one body region or function.
/// Chemical modulation is applied at the bundle level.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FiberBundle {
    /// Human-readable name ("left_arm", "vocal_tract", "gaze").
    pub name: String,

    /// Tracts in this bundle, indexed by kind.
    pub tracts: Vec<FiberTract>,
}

impl FiberBundle {
    /// Create a new empty bundle.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tracts: Vec::new(),
        }
    }

    /// Add a tract to this bundle.
    pub fn add_tract(&mut self, tract: FiberTract) {
        self.tracts.push(tract);
    }

    /// Get first tract of a given kind (if present).
    pub fn tract(&self, kind: FiberTractKind) -> Option<&FiberTract> {
        self.tracts.iter().find(|t| t.kind == kind)
    }

    /// Get mutable reference to first tract of a given kind.
    pub fn tract_mut(&mut self, kind: FiberTractKind) -> Option<&mut FiberTract> {
        self.tracts.iter_mut().find(|t| t.kind == kind)
    }

    /// All motor (efferent) tracts in this bundle.
    pub fn motor_tracts(&self) -> impl Iterator<Item = &FiberTract> {
        self.tracts.iter().filter(|t| t.kind.is_efferent())
    }

    /// All sensory (afferent) tracts in this bundle.
    pub fn sensory_tracts(&self) -> impl Iterator<Item = &FiberTract> {
        self.tracts.iter().filter(|t| t.kind.is_afferent())
    }

    /// Whether any tract in this bundle is actively transmitting.
    pub fn is_active(&self) -> bool {
        self.tracts.iter().any(|t| t.is_active())
    }

    /// Total activity across all tracts.
    pub fn total_activity(&self) -> u64 {
        self.tracts.iter().map(|t| t.activity_level()).sum()
    }

    /// Number of tracts in this bundle.
    pub fn tract_count(&self) -> usize {
        self.tracts.len()
    }

    // === Chemical Modulation ===
    //
    // These methods apply broadcast chemical effects to all tracts in
    // the bundle. Biology: chemistry is systemic, not per-fiber.

    /// Adrenaline dump: increase motor gain, decrease sensory sensitivity.
    ///
    /// `intensity`: 0-255 chemical concentration.
    /// Fight-or-flight: stronger output, reduced pain awareness.
    pub fn apply_adrenaline(&mut self, intensity: u8) {
        let boost = intensity / 4; // 0-63 range
        for tract in &mut self.tracts {
            if tract.kind.is_efferent() {
                tract.gain = tract.gain.saturating_add(boost);
            } else if matches!(
                tract.kind,
                FiberTractKind::NociceptiveFast | FiberTractKind::NociceptiveSlow
            ) {
                // Reduce pain sensitivity under adrenaline
                tract.sensitivity = tract.sensitivity.saturating_sub(boost);
            }
        }
    }

    /// Endorphin release: reduce nociceptive sensitivity across bundle.
    ///
    /// `intensity`: 0-255 chemical concentration.
    /// Natural painkiller — gates pain at the fiber level.
    pub fn apply_endorphin(&mut self, intensity: u8) {
        let reduction = intensity / 3; // 0-85 range
        for tract in &mut self.tracts {
            if matches!(
                tract.kind,
                FiberTractKind::NociceptiveFast | FiberTractKind::NociceptiveSlow
            ) {
                tract.sensitivity = tract.sensitivity.saturating_sub(reduction);
            }
        }
    }

    /// Cortisol (sustained stress): increase jitter, decrease endurance.
    ///
    /// `intensity`: 0-255 chemical concentration.
    /// Chronic stress degrades signal quality and stamina.
    pub fn apply_cortisol(&mut self, intensity: u8) {
        let degradation = intensity / 8; // 0-31 range (gradual)
        for tract in &mut self.tracts {
            tract.jitter = tract.jitter.saturating_add(degradation);
            tract.endurance = tract.endurance.saturating_sub(degradation / 2);
        }
    }

    /// GABA (inhibition): reduce gain across all tracts.
    ///
    /// `intensity`: 0-255 chemical concentration.
    /// Calming — lowers overall neural transmission.
    pub fn apply_gaba(&mut self, intensity: u8) {
        let reduction = intensity / 4;
        for tract in &mut self.tracts {
            tract.gain = tract.gain.saturating_sub(reduction);
        }
    }

    /// Reset chemical modulation to neutral defaults.
    ///
    /// Useful when a chemical effect wears off and you want
    /// to return to baseline before applying the next state.
    pub fn reset_to_baseline(&mut self) {
        for tract in &mut self.tracts {
            if tract.kind.is_efferent() {
                tract.gain = 160; // motor default
            } else {
                tract.gain = 100; // sensory default
            }
            tract.sensitivity = 128;
            tract.jitter = 128;
            tract.endurance = 128;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_bundle() -> FiberBundle {
        let mut bundle = FiberBundle::new("test_limb");
        bundle.add_tract(FiberTract::new_motor(FiberTractKind::MotorSkeletal, 8));
        bundle.add_tract(FiberTract::new_sensory(FiberTractKind::Proprioceptive, 8));
        bundle.add_tract(FiberTract::new_sensory(FiberTractKind::NociceptiveFast, 4));
        bundle.add_tract(FiberTract::new_sensory(FiberTractKind::NociceptiveSlow, 4));
        bundle
    }

    #[test]
    fn bundle_creation() {
        let bundle = make_test_bundle();
        assert_eq!(bundle.name, "test_limb");
        assert_eq!(bundle.tract_count(), 4);
        assert_eq!(bundle.motor_tracts().count(), 1);
        assert_eq!(bundle.sensory_tracts().count(), 3);
    }

    #[test]
    fn bundle_tract_lookup() {
        let bundle = make_test_bundle();
        assert!(bundle.tract(FiberTractKind::MotorSkeletal).is_some());
        assert!(bundle.tract(FiberTractKind::Proprioceptive).is_some());
        assert!(bundle.tract(FiberTractKind::NociceptiveFast).is_some());
        assert!(bundle.tract(FiberTractKind::MotorSpindle).is_none());
    }

    #[test]
    fn adrenaline_boosts_motor_gates_pain() {
        let mut bundle = make_test_bundle();

        let motor_gain_before = bundle.tract(FiberTractKind::MotorSkeletal).unwrap().gain;
        let noci_sens_before = bundle
            .tract(FiberTractKind::NociceptiveFast)
            .unwrap()
            .sensitivity;

        bundle.apply_adrenaline(200);

        let motor_gain_after = bundle.tract(FiberTractKind::MotorSkeletal).unwrap().gain;
        let noci_sens_after = bundle
            .tract(FiberTractKind::NociceptiveFast)
            .unwrap()
            .sensitivity;

        assert!(motor_gain_after > motor_gain_before, "motor gain should increase");
        assert!(
            noci_sens_after < noci_sens_before,
            "nociceptive sensitivity should decrease"
        );
    }

    #[test]
    fn endorphin_reduces_pain_only() {
        let mut bundle = make_test_bundle();

        let proprio_sens_before = bundle
            .tract(FiberTractKind::Proprioceptive)
            .unwrap()
            .sensitivity;
        let noci_sens_before = bundle
            .tract(FiberTractKind::NociceptiveFast)
            .unwrap()
            .sensitivity;

        bundle.apply_endorphin(200);

        let proprio_sens_after = bundle
            .tract(FiberTractKind::Proprioceptive)
            .unwrap()
            .sensitivity;
        let noci_sens_after = bundle
            .tract(FiberTractKind::NociceptiveFast)
            .unwrap()
            .sensitivity;

        assert_eq!(
            proprio_sens_after, proprio_sens_before,
            "proprioception should be unaffected"
        );
        assert!(
            noci_sens_after < noci_sens_before,
            "nociceptive sensitivity should decrease"
        );
    }

    #[test]
    fn cortisol_degrades_signal_quality() {
        let mut bundle = make_test_bundle();

        let jitter_before = bundle.tract(FiberTractKind::MotorSkeletal).unwrap().jitter;

        bundle.apply_cortisol(200);

        let jitter_after = bundle.tract(FiberTractKind::MotorSkeletal).unwrap().jitter;

        assert!(jitter_after > jitter_before, "jitter should increase under cortisol");
    }
}
