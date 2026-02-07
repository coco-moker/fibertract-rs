//! LimbProfile — preset configurations for body regions.
//!
//! Each body region has different fiber compositions, sensitivities,
//! and motor capabilities. A hand has fine motor control and high
//! mechanoreceptive density. A leg has high strength and proprioception.
//! Vocal cords have extreme motor precision and minimal pain fibers.
//!
//! Profiles create pre-configured FiberBundles with biologically
//! appropriate tract dimensions and property defaults.

use crate::bundle::FiberBundle;
use crate::tract::{FiberTract, FiberTractKind, ReceptorMode};

/// A profile defining the fiber composition of a body region.
///
/// Specifies which tract kinds are present, their dimensions,
/// and initial property overrides from defaults.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LimbProfile {
    /// Profile name (matches bundle name).
    pub name: String,

    /// Tract specifications for this profile.
    pub tracts: Vec<TractSpec>,
}

/// Specification for a single tract within a profile.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TractSpec {
    pub kind: FiberTractKind,
    pub dim: usize,
    /// Override defaults (None = use tract defaults).
    pub conductivity: Option<u8>,
    pub jitter: Option<u8>,
    pub gain: Option<u8>,
    pub sensitivity: Option<u8>,
    pub endurance: Option<u8>,
    pub elasticity: Option<u8>,
    pub strength: Option<u8>,
    /// Receptor mode: phasic (default) or tonic.
    pub receptor_mode: Option<ReceptorMode>,
}

impl TractSpec {
    /// Create a spec with all defaults for a given kind and dimension.
    pub fn new(kind: FiberTractKind, dim: usize) -> Self {
        Self {
            kind,
            dim,
            conductivity: None,
            jitter: None,
            gain: None,
            sensitivity: None,
            endurance: None,
            elasticity: None,
            strength: None,
            receptor_mode: None,
        }
    }

    /// Build the actual FiberTract from this spec.
    fn build(&self) -> FiberTract {
        let mut tract = if self.kind.is_efferent() {
            FiberTract::new_motor(self.kind, self.dim)
        } else {
            FiberTract::new_sensory(self.kind, self.dim)
        };

        if let Some(v) = self.conductivity { tract.conductivity = v; }
        if let Some(v) = self.jitter { tract.jitter = v; }
        if let Some(v) = self.gain { tract.gain = v; }
        if let Some(v) = self.sensitivity { tract.sensitivity = v; }
        if let Some(v) = self.endurance { tract.endurance = v; }
        if let Some(v) = self.elasticity { tract.elasticity = v; }
        if let Some(v) = self.strength { tract.strength = v; }
        if let Some(v) = self.receptor_mode { tract.receptor_mode = v; }

        tract
    }
}

impl LimbProfile {
    /// Build a FiberBundle from this profile.
    pub fn build(&self) -> FiberBundle {
        let mut bundle = FiberBundle::new(&self.name);
        for spec in &self.tracts {
            bundle.add_tract(spec.build());
        }
        bundle
    }

    // === Preset Profiles ===

    /// Hand — high dexterity, dense mechanoreception, fine motor control.
    ///
    /// Highest density of mechanoreceptors in the body. Fine motor with
    /// low jitter. Moderate pain fibers. High elasticity for rapid movements.
    pub fn hand(side: &str) -> Self {
        let name = format!("{side}_hand");
        Self {
            name,
            tracts: vec![
                // Motor: fine control, many channels
                TractSpec {
                    kind: FiberTractKind::MotorSkeletal,
                    dim: 32,
                    gain: Some(150),       // moderate amplification
                    jitter: Some(40),      // very clean signals (dexterous)
                    elasticity: Some(220), // fast tracking (quick fingers)
                    strength: Some(100),   // moderate strength
                    ..TractSpec::new(FiberTractKind::MotorSkeletal, 32)
                },
                // Muscle tone
                TractSpec {
                    kind: FiberTractKind::MotorSpindle,
                    dim: 16,
                    ..TractSpec::new(FiberTractKind::MotorSpindle, 16)
                },
                // Touch: very dense, high sensitivity
                TractSpec {
                    kind: FiberTractKind::Mechanoreceptive,
                    dim: 64,                        // highest density
                    sensitivity: Some(230),         // very sensitive
                    gain: Some(110),                // slight attenuation
                    jitter: Some(30),               // clean
                    ..TractSpec::new(FiberTractKind::Mechanoreceptive, 64)
                },
                // Proprioception: fine position sense
                TractSpec {
                    kind: FiberTractKind::Proprioceptive,
                    dim: 24,
                    sensitivity: Some(200),
                    ..TractSpec::new(FiberTractKind::Proprioceptive, 24)
                },
                // Pain: moderate
                TractSpec::new(FiberTractKind::NociceptiveFast, 8),
                TractSpec::new(FiberTractKind::NociceptiveSlow, 4),
            ],
        }
    }

    /// Arm — moderate dexterity, good strength, balanced sensory.
    pub fn arm(side: &str) -> Self {
        let name = format!("{side}_arm");
        Self {
            name,
            tracts: vec![
                TractSpec {
                    kind: FiberTractKind::MotorSkeletal,
                    dim: 16,
                    gain: Some(180),       // strong amplification
                    strength: Some(160),   // good strength
                    endurance: Some(150),
                    ..TractSpec::new(FiberTractKind::MotorSkeletal, 16)
                },
                TractSpec::new(FiberTractKind::MotorSpindle, 8),
                TractSpec {
                    kind: FiberTractKind::Mechanoreceptive,
                    dim: 16,
                    sensitivity: Some(150),
                    ..TractSpec::new(FiberTractKind::Mechanoreceptive, 16)
                },
                TractSpec {
                    kind: FiberTractKind::Proprioceptive,
                    dim: 16,
                    sensitivity: Some(180),
                    ..TractSpec::new(FiberTractKind::Proprioceptive, 16)
                },
                TractSpec::new(FiberTractKind::NociceptiveFast, 8),
                TractSpec::new(FiberTractKind::NociceptiveSlow, 4),
                TractSpec::new(FiberTractKind::Interoceptive, 4),
            ],
        }
    }

    /// Leg — high strength, high endurance, coarser motor control.
    pub fn leg(side: &str) -> Self {
        let name = format!("{side}_leg");
        Self {
            name,
            tracts: vec![
                TractSpec {
                    kind: FiberTractKind::MotorSkeletal,
                    dim: 12,
                    gain: Some(200),       // high amplification (power)
                    strength: Some(220),   // very strong
                    endurance: Some(200),  // high endurance (walking)
                    jitter: Some(160),     // coarser control
                    elasticity: Some(140), // slower than hands
                    ..TractSpec::new(FiberTractKind::MotorSkeletal, 12)
                },
                TractSpec::new(FiberTractKind::MotorSpindle, 8),
                TractSpec {
                    kind: FiberTractKind::Mechanoreceptive,
                    dim: 16,
                    sensitivity: Some(120), // moderate (feet are less sensitive)
                    ..TractSpec::new(FiberTractKind::Mechanoreceptive, 16)
                },
                TractSpec {
                    kind: FiberTractKind::Proprioceptive,
                    dim: 20,
                    sensitivity: Some(220), // very important for balance
                    ..TractSpec::new(FiberTractKind::Proprioceptive, 20)
                },
                TractSpec::new(FiberTractKind::NociceptiveFast, 8),
                TractSpec::new(FiberTractKind::NociceptiveSlow, 4),
                TractSpec::new(FiberTractKind::Interoceptive, 8), // fatigue awareness
            ],
        }
    }

    /// Vocal tract — extreme motor precision, minimal sensory.
    ///
    /// The vocal cords require incredibly precise, fast motor control
    /// with very low jitter. Proprioception is critical for pitch control.
    /// Very few pain fibers (you don't feel individual vocal fold movements).
    pub fn vocal_tract() -> Self {
        Self {
            name: "vocal_tract".into(),
            tracts: vec![
                TractSpec {
                    kind: FiberTractKind::MotorSkeletal,
                    dim: 24,
                    gain: Some(140),       // moderate (precision > power)
                    jitter: Some(20),      // extremely clean
                    elasticity: Some(250), // near-instant tracking
                    strength: Some(60),    // low force needed
                    endurance: Some(180),  // can talk for hours
                    ..TractSpec::new(FiberTractKind::MotorSkeletal, 24)
                },
                TractSpec {
                    kind: FiberTractKind::Proprioceptive,
                    dim: 16,
                    sensitivity: Some(240), // critical for pitch
                    ..TractSpec::new(FiberTractKind::Proprioceptive, 16)
                },
                // Minimal pain and touch
                TractSpec::new(FiberTractKind::NociceptiveFast, 2),
                TractSpec::new(FiberTractKind::Interoceptive, 4), // throat fatigue
            ],
        }
    }

    /// Gaze control — eye movement, rapid saccades.
    ///
    /// Fastest motor response in the body. Very low latency,
    /// very low jitter. No pain fibers in eye muscles.
    pub fn gaze() -> Self {
        Self {
            name: "gaze".into(),
            tracts: vec![
                TractSpec {
                    kind: FiberTractKind::MotorSkeletal,
                    dim: 12,
                    gain: Some(140),
                    jitter: Some(15),      // cleanest signals in the body
                    elasticity: Some(255), // instant
                    strength: Some(40),    // tiny muscles
                    endurance: Some(220),  // saccades all day
                    ..TractSpec::new(FiberTractKind::MotorSkeletal, 12)
                },
                TractSpec {
                    kind: FiberTractKind::Proprioceptive,
                    dim: 12,
                    sensitivity: Some(250), // extreme precision
                    ..TractSpec::new(FiberTractKind::Proprioceptive, 12)
                },
                // No pain fibers in extraocular muscles
            ],
        }
    }

    /// Torso — core stability, high endurance, deep pain awareness.
    pub fn torso() -> Self {
        Self {
            name: "torso".into(),
            tracts: vec![
                TractSpec {
                    kind: FiberTractKind::MotorSkeletal,
                    dim: 8,
                    gain: Some(170),
                    strength: Some(200),
                    endurance: Some(230), // postural muscles never stop
                    jitter: Some(160),    // coarse control
                    ..TractSpec::new(FiberTractKind::MotorSkeletal, 8)
                },
                TractSpec::new(FiberTractKind::MotorSpindle, 8),
                TractSpec {
                    kind: FiberTractKind::Mechanoreceptive,
                    dim: 8,
                    sensitivity: Some(100), // low density
                    ..TractSpec::new(FiberTractKind::Mechanoreceptive, 8)
                },
                TractSpec {
                    kind: FiberTractKind::Proprioceptive,
                    dim: 12,
                    sensitivity: Some(180),
                    ..TractSpec::new(FiberTractKind::Proprioceptive, 12)
                },
                TractSpec::new(FiberTractKind::NociceptiveFast, 4),
                TractSpec::new(FiberTractKind::NociceptiveSlow, 8), // deep ache
                TractSpec {
                    kind: FiberTractKind::Interoceptive,
                    dim: 16,                // high visceral awareness
                    sensitivity: Some(180), // gut feelings
                    ..TractSpec::new(FiberTractKind::Interoceptive, 16)
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hand_profile_builds() {
        let profile = LimbProfile::hand("left");
        let bundle = profile.build();

        assert_eq!(bundle.name, "left_hand");
        assert!(bundle.tract_count() >= 5);
        assert!(bundle.motor_tracts().count() >= 1);
        assert!(bundle.sensory_tracts().count() >= 3);

        // Hand should have highest mechanoreceptive density
        let mechano = bundle.tract(FiberTractKind::Mechanoreceptive).unwrap();
        assert_eq!(mechano.dim, 64);
        assert_eq!(mechano.sensitivity, 230);
    }

    #[test]
    fn vocal_tract_is_precise() {
        let profile = LimbProfile::vocal_tract();
        let bundle = profile.build();

        let motor = bundle.tract(FiberTractKind::MotorSkeletal).unwrap();
        assert!(motor.jitter < 30, "vocal motor jitter should be very low");
        assert!(motor.elasticity > 240, "vocal elasticity should be near-instant");
    }

    #[test]
    fn gaze_has_no_pain() {
        let profile = LimbProfile::gaze();
        let bundle = profile.build();

        assert!(
            bundle.tract(FiberTractKind::NociceptiveFast).is_none(),
            "eye muscles have no pain fibers"
        );
        assert!(
            bundle.tract(FiberTractKind::NociceptiveSlow).is_none(),
            "eye muscles have no pain fibers"
        );
    }

    #[test]
    fn leg_stronger_than_hand() {
        let hand = LimbProfile::hand("right").build();
        let leg = LimbProfile::leg("right").build();

        let hand_motor = hand.tract(FiberTractKind::MotorSkeletal).unwrap();
        let leg_motor = leg.tract(FiberTractKind::MotorSkeletal).unwrap();

        assert!(
            leg_motor.strength > hand_motor.strength,
            "legs should be stronger than hands"
        );
        assert!(
            hand_motor.jitter < leg_motor.jitter,
            "hands should have less jitter than legs"
        );
    }

    #[test]
    fn all_profiles_build_successfully() {
        let profiles = vec![
            LimbProfile::hand("left"),
            LimbProfile::hand("right"),
            LimbProfile::arm("left"),
            LimbProfile::arm("right"),
            LimbProfile::leg("left"),
            LimbProfile::leg("right"),
            LimbProfile::vocal_tract(),
            LimbProfile::gaze(),
            LimbProfile::torso(),
        ];

        for profile in profiles {
            let bundle = profile.build();
            assert!(!bundle.name.is_empty());
            assert!(bundle.tract_count() > 0);
        }
    }
}
