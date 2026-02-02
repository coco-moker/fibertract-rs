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

    if active {
        tract.lifetime_activations = tract.lifetime_activations.saturating_add(1);

        // Update rolling density: push toward 255
        tract.recent_density = density.saturating_add(
            (255u16.saturating_sub(density as u16) / 16) as u8
        ).max(density.saturating_add(1));

        // === Active adaptations ===

        // Myelination: conductivity improves with use
        tract.conductivity = tract
            .conductivity
            .saturating_add(config.myelination_rate);

        // Practice: jitter decreases with consistent use
        if density > 128 {
            // Only if sustained activity (not just a single spike)
            tract.jitter = tract
                .jitter
                .saturating_sub(config.jitter_improvement_rate);
        }

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
        // Update rolling density: decay toward 0
        tract.recent_density = density.saturating_sub(
            (density as u16 / 16).max(1) as u8
        );

        // === Idle adaptations ===

        // Recovery: fatigue decreases at rest
        let effective_recovery = config.recovery_rate.saturating_add(tract.endurance / 64);
        tract.fatigue = tract.fatigue.saturating_sub(effective_recovery);

        // Demyelination: conductivity degrades without use
        if density < config.idle_threshold {
            tract.conductivity = tract
                .conductivity
                .saturating_sub(config.demyelination_rate);
        }

        // Jitter increase: precision decays without practice
        if density < config.idle_threshold {
            tract.jitter = tract
                .jitter
                .saturating_add(config.jitter_decay_rate);
        }

        // Atrophy: strength decays after prolonged disuse
        if density == 0 {
            // Only atrophy if completely idle for a while
            // (lifetime_activations > 0 means it was used at some point)
            if tract.lifetime_activations > 0 {
                tract.strength = tract
                    .strength
                    .saturating_sub(config.atrophy_rate);
            }
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
    fn active_tract_myelinates() {
        let mut tract = FiberTract::new_motor(FiberTractKind::MotorSkeletal, 2);
        let initial_cond = tract.conductivity;

        // Make it active by putting signals in
        tract.motor_signals[0] = Signal { polarity: 1, magnitude: 100 };

        let config = AdaptationConfig::default();
        for _ in 0..10 {
            adapt_tract(&mut tract, &config);
        }

        assert!(
            tract.conductivity > initial_cond,
            "conductivity should improve with use"
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
