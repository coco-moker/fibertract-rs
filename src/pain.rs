//! Pain and nociception — the alarm system.
//!
//! Pain is not a sensation. Pain is a _decision_ the nervous system makes
//! based on nociceptive input, context, and chemical state. The same signal
//! can be pain or just information depending on the system's state.
//!
//! This module provides the data structures for pain events and their sources.
//! Pain gating (endorphins, adrenaline, attention) happens at the bundle level
//! via chemical modulation and sensitivity thresholds.

use crate::tract::FiberTractKind;

/// A pain event detected from nociceptive fiber activity.
///
/// Created when nociceptive tracts exceed their sensitivity threshold
/// after chemical gating. The event carries enough information for
/// the brain to decide what to do about it.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PainEvent {
    /// Which bundle originated this pain.
    pub bundle_name: String,

    /// The source/type of pain.
    pub source: PainSource,

    /// Intensity after all gating (0-255). This is the PERCEIVED intensity,
    /// not the raw stimulus. Chemical state, attention, and prior adaptation
    /// have already shaped this value.
    pub intensity: u8,

    /// Onset sharpness: how fast the pain signal rose.
    /// High = sudden injury. Low = gradual ache.
    /// Computed from delta between current and previous nociceptive signals.
    pub onset: u8,

    /// Duration in ticks this pain source has been continuously active.
    /// Short = acute. Long = chronic. Affects urgency weighting.
    pub duration_ticks: u32,

    /// Whether this pain is habituating (decreasing despite constant stimulus).
    /// True = the system is adapting. False = pain is sustained or increasing.
    pub habituating: bool,
}

/// Source classification of pain — what biological process generated it.
///
/// Maps to nociceptive fiber types and interoceptive signals.
/// The brain uses this to select response strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum PainSource {
    /// Sharp, well-localized pain (Aδ fibers). Fast onset, fast offset.
    /// Response: withdraw reflex, immediate attention.
    Sharp = 0,

    /// Burning, diffuse pain (C fibers). Slow onset, lingers.
    /// Response: guarding behavior, sustained attention.
    Burning = 1,

    /// Aching, deep pain (C fibers, deep tissue). Dull, poorly localized.
    /// Response: posture change, rest-seeking.
    Aching = 2,

    /// Visceral distress (interoceptive C fibers). Internal discomfort.
    /// Response: behavioral change (eat, rest, cool down).
    Visceral = 3,

    /// Fatigue-pain (interoceptive, metabolic). Overexertion signal.
    /// Response: reduce effort, seek recovery.
    Fatigue = 4,
}

impl PainSource {
    /// Which fiber tract kind typically generates this pain type.
    pub fn primary_tract(self) -> FiberTractKind {
        match self {
            Self::Sharp => FiberTractKind::NociceptiveFast,
            Self::Burning | Self::Aching => FiberTractKind::NociceptiveSlow,
            Self::Visceral | Self::Fatigue => FiberTractKind::Interoceptive,
        }
    }

    /// Urgency level (0-255). How quickly the brain should respond.
    /// Sharp pain = immediate. Fatigue = can wait.
    pub fn urgency(self) -> u8 {
        match self {
            Self::Sharp => 240,
            Self::Burning => 180,
            Self::Visceral => 140,
            Self::Aching => 100,
            Self::Fatigue => 60,
        }
    }

    /// Construct from u8 ordinal.
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Sharp),
            1 => Some(Self::Burning),
            2 => Some(Self::Aching),
            3 => Some(Self::Visceral),
            4 => Some(Self::Fatigue),
            _ => None,
        }
    }
}

impl PainEvent {
    /// Whether this pain requires immediate attention (high urgency + high intensity).
    pub fn is_urgent(&self) -> bool {
        let urgency = self.source.urgency();
        // Both intensity and urgency must be high
        self.intensity > 128 && urgency > 160
    }

    /// Whether this pain is chronic (long duration, not habituating).
    pub fn is_chronic(&self) -> bool {
        self.duration_ticks > 1000 && !self.habituating
    }

    /// Combined salience score (0-255) for attention allocation.
    /// Weighs intensity, urgency, onset, and novelty (inverse habituation).
    pub fn salience(&self) -> u8 {
        let urgency = self.source.urgency() as u16;
        let intensity = self.intensity as u16;
        let onset_weight = self.onset as u16;
        let novelty = if self.habituating { 64u16 } else { 192u16 };

        // Weighted average: intensity*3 + urgency*2 + onset*1 + novelty*2
        let score = (intensity * 3 + urgency * 2 + onset_weight + novelty * 2) / 8;
        score.min(255) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pain_source_tracts() {
        assert_eq!(
            PainSource::Sharp.primary_tract(),
            FiberTractKind::NociceptiveFast
        );
        assert_eq!(
            PainSource::Burning.primary_tract(),
            FiberTractKind::NociceptiveSlow
        );
        assert_eq!(
            PainSource::Fatigue.primary_tract(),
            FiberTractKind::Interoceptive
        );
    }

    #[test]
    fn urgency_ordering() {
        assert!(PainSource::Sharp.urgency() > PainSource::Burning.urgency());
        assert!(PainSource::Burning.urgency() > PainSource::Aching.urgency());
        assert!(PainSource::Visceral.urgency() > PainSource::Fatigue.urgency());
    }

    #[test]
    fn sharp_high_intensity_is_urgent() {
        let event = PainEvent {
            bundle_name: "left_arm".into(),
            source: PainSource::Sharp,
            intensity: 200,
            onset: 220,
            duration_ticks: 5,
            habituating: false,
        };
        assert!(event.is_urgent());
    }

    #[test]
    fn low_ache_not_urgent() {
        let event = PainEvent {
            bundle_name: "lower_back".into(),
            source: PainSource::Aching,
            intensity: 80,
            onset: 20,
            duration_ticks: 5000,
            habituating: false,
        };
        assert!(!event.is_urgent());
        assert!(event.is_chronic());
    }

    #[test]
    fn habituation_reduces_salience() {
        let fresh = PainEvent {
            bundle_name: "test".into(),
            source: PainSource::Burning,
            intensity: 150,
            onset: 100,
            duration_ticks: 10,
            habituating: false,
        };
        let habituated = PainEvent {
            habituating: true,
            ..fresh.clone()
        };

        assert!(fresh.salience() > habituated.salience());
    }
}
