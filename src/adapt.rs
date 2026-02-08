//! Use-dependent adaptation — connective tissue remodeling.
//!
//! Tracts don't learn (no plasticity, no weight changes). They ADAPT:
//! properties change based on usage patterns over time.
//!
//! - Active tracts: conductivity improves, jitter decreases, strength grows
//! - Idle tracts: conductivity degrades, strength atrophies
//! - Overused tracts: fatigue accumulates (tempered by endurance)
//! - Recovery: fatigue decreases during rest (modulated by endurance)
//!
//! This is connective tissue biology, not neural learning.
//! Muscles grow with use and atrophy without. Nerves myelinate with
//! sustained activity and demyelinate without. Same principle.

use crate::tract::FiberTract;
use crate::bundle::FiberBundle;

/// Adaptation rate constants. All u8 to avoid float.
/// These control how fast properties change per tick.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AdaptationConfig {
    /// How fast conductivity improves with use (per active tick).
    pub myelination_rate: u8,

    /// How fast conductivity degrades without use (per idle tick).
    pub demyelination_rate: u8,

    /// How fast jitter decreases with consistent use (practice effect).
    pub jitter_improvement_rate: u8,

    /// How fast jitter increases during disuse (skill decay).
    pub jitter_decay_rate: u8,

    /// How fast fatigue accumulates during active transmission.
    pub fatigue_rate: u8,

    /// How fast fatigue recovers during rest.
    pub recovery_rate: u8,

    /// How fast strength grows with sustained effort.
    pub strengthening_rate: u8,

    /// How fast strength decays during disuse (atrophy).
    pub atrophy_rate: u8,

    /// Ticks of inactivity before atrophy begins.
    pub atrophy_delay: u8,

    /// Activity density threshold below which a tract is "idle" (0-255).
    pub idle_threshold: u8,

    /// How fast sensitivity increases with repeated input stimulation.
    /// Biological analog: peripheral sensitization — repeated sub-threshold
    /// stimulation lowers the activation threshold.
    pub sensitization_rate: u8,

    /// How fast sensitivity decreases without input stimulation.
    /// Biological analog: desensitization / denervation.
    pub desensitization_rate: u8,
}

impl Default for AdaptationConfig {
    fn default() -> Self {
        Self {
            myelination_rate: 1,
            demyelination_rate: 1,
            jitter_improvement_rate: 1,
            jitter_decay_rate: 1,
            fatigue_rate: 2,
            recovery_rate: 3, // recovery is slightly faster than fatigue
            strengthening_rate: 1,
            atrophy_rate: 1,
            atrophy_delay: 50,   // ~50 idle ticks before atrophy
            idle_threshold: 10,  // density below 10 = idle
            sensitization_rate: 1,
            desensitization_rate: 1,
        }
    }
}

/// Tick a single tract's adaptation.
///
/// Call this once per simulation tick for each tract.
/// Updates conductivity, jitter, fatigue, strength based on current activity.
pub fn adapt_tract(tract: &mut FiberTract, config: &AdaptationConfig) {
    let active = tract.is_active();
    let density = tract.recent_density;
    let input_density = tract.input_density;

    // === Output-driven adaptation (fatigue, strength, output density) ===

    if active {
        tract.lifetime_activations = tract.lifetime_activations.saturating_add(1);

        // Update rolling output density: push toward 255
        tract.recent_density = density.saturating_add(
            (255u16.saturating_sub(density as u16) / 16) as u8
        ).max(density.saturating_add(1));

        // Fatigue: accumulates during use, tempered by endurance
        let effective_fatigue_rate = if config.fatigue_rate as u16 > tract.endurance as u16 / 32 {
            config.fatigue_rate - (tract.endurance / 32) as u8
        } else {
            0
        };
        tract.fatigue = tract.fatigue.saturating_add(effective_fatigue_rate);

        // Strength: grows with sustained effort against resistance (high fatigue = training)
        if tract.fatigue > 128 && density > 100 {
            tract.strength = tract
                .strength
                .saturating_add(config.strengthening_rate);
        }
    } else {
        // Update rolling output density: decay toward 0
        tract.recent_density = density.saturating_sub(
            (density as u16 / 16).max(1) as u8
        );

        // Recovery: fatigue decreases at rest
        let effective_recovery = config.recovery_rate.saturating_add(tract.endurance / 64);
        tract.fatigue = tract.fatigue.saturating_sub(effective_recovery);

        // Atrophy: strength decays after prolonged disuse
        if density == 0 && tract.lifetime_activations > 0 {
            tract.strength = tract
                .strength
                .saturating_sub(config.atrophy_rate);
        }
    }

    // === Input-driven adaptation (myelination, jitter, sensitivity) ===
    //
    // A tract that receives repeated stimulation adapts even before it can
    // fully conduct. This is biological: repeated stimulation triggers
    // myelination and lowers activation threshold (sensitization), even
    // when the nerve can't yet propagate the signal end-to-end.

    let input_active = input_density > config.idle_threshold;

    if input_active {
        // Myelination: conductivity improves with stimulation
        tract.conductivity = tract
            .conductivity
            .saturating_add(config.myelination_rate);

        // Sensitization: sensitivity increases (recruitment threshold drops)
        tract.sensitivity = tract
            .sensitivity
            .saturating_add(config.sensitization_rate);

        // Practice: jitter decreases with sustained stimulation
        if input_density > 128 {
            tract.jitter = tract
                .jitter
                .saturating_sub(config.jitter_improvement_rate);
        }
    } else {
        // Demyelination: conductivity degrades without stimulation
        if input_density == 0 && density < config.idle_threshold {
            tract.conductivity = tract
                .conductivity
                .saturating_sub(config.demyelination_rate);
        }

        // Desensitization: sensitivity decreases (threshold rises)
        if input_density == 0 {
            tract.sensitivity = tract
                .sensitivity
                .saturating_sub(config.desensitization_rate);
        }

        // Jitter increase: precision decays without practice
        if input_density == 0 && density < config.idle_threshold {
            tract.jitter = tract
                .jitter
                .saturating_add(config.jitter_decay_rate);
        }
    }
}

/// Tick adaptation for all tracts in a bundle.
pub fn adapt_bundle(bundle: &mut FiberBundle, config: &AdaptationConfig) {
    for tract in &mut bundle.tracts {
        adapt_tract(tract, config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tract::FiberTractKind;
    use ternary_signal::Signal;

    #[test]
    fn stimulated_tract_myelinates() {
        let mut tract = FiberTract::new_motor(FiberTractKind::MotorSkeletal, 2);
        let initial_cond = tract.conductivity;
        let initial_sens = tract.sensitivity;

        // Transmit through the tract to build input_density naturally.
        // Even if output is gated, the input stimulation drives adaptation.
        let input = [Signal { polarity: 1, magnitude: 100 }, Signal::default()];
        let config = AdaptationConfig::default();
        for i in 0..30 {
            tract.transmit_motor(&input, i);
            adapt_tract(&mut tract, &config);
        }

        assert!(
            tract.conductivity > initial_cond,
            "conductivity should improve with stimulation: {} vs {}",
            tract.conductivity, initial_cond,
        );
        assert!(
            tract.sensitivity > initial_sens,
            "sensitivity should increase with stimulation: {} vs {}",
            tract.sensitivity, initial_sens,
        );
    }

    #[test]
    fn idle_tract_atrophies() {
        let mut tract = FiberTract::new_motor(FiberTractKind::MotorSkeletal, 2);
        tract.lifetime_activations = 100; // was used before
        tract.strength = 200;
        tract.recent_density = 0;

        let config = AdaptationConfig::default();
        for _ in 0..50 {
            adapt_tract(&mut tract, &config);
        }

        assert!(tract.strength < 200, "strength should decay with disuse");
    }

    #[test]
    fn fatigue_recovers_at_rest() {
        let mut tract = FiberTract::new_motor(FiberTractKind::MotorSkeletal, 2);
        tract.fatigue = 200; // very tired

        let config = AdaptationConfig::default();
        for _ in 0..30 {
            adapt_tract(&mut tract, &config);
        }

        assert!(tract.fatigue < 200, "fatigue should decrease during rest");
    }

    #[test]
    fn endurance_slows_fatigue() {
        let mut tract_fragile = FiberTract::new_motor(FiberTractKind::MotorSkeletal, 2);
        tract_fragile.endurance = 0;
        tract_fragile.motor_signals[0] = Signal { polarity: 1, magnitude: 100 };

        let mut tract_tough = FiberTract::new_motor(FiberTractKind::MotorSkeletal, 2);
        tract_tough.endurance = 255;
        tract_tough.motor_signals[0] = Signal { polarity: 1, magnitude: 100 };

        let config = AdaptationConfig::default();
        for _ in 0..20 {
            adapt_tract(&mut tract_fragile, &config);
            adapt_tract(&mut tract_tough, &config);
        }

        assert!(
            tract_fragile.fatigue > tract_tough.fatigue,
            "fragile tract should fatigue faster than tough one"
        );
    }

    #[test]
    fn density_tracks_activity() {
        let mut tract = FiberTract::new_motor(FiberTractKind::MotorSkeletal, 2);
        tract.motor_signals[0] = Signal { polarity: 1, magnitude: 100 };

        let config = AdaptationConfig::default();

        // Build up density
        for _ in 0..20 {
            adapt_tract(&mut tract, &config);
        }

        let active_density = tract.recent_density;
        assert!(active_density > 0, "density should increase with activity");

        // Stop activity, let density decay
        tract.motor_signals[0] = Signal::default();
        for _ in 0..40 {
            adapt_tract(&mut tract, &config);
        }

        assert!(
            tract.recent_density < active_density,
            "density should decay when idle"
        );
    }
}
