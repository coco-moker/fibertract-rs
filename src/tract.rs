//! FiberTract — a single labeled-line nerve tract.
//!
//! The tract kind defines what signals MEAN. Motor tracts carry Signal
//! (brain's ternary format). Sensory tracts carry i32 (wide dynamic range,
//! Weber-law quantized). Physical properties (all u8) shape transmission.

use ternary_signal::Signal;

use crate::weber::weber_quantize;

/// Receptor adaptation mode — phasic vs tonic.
///
/// Biological receptors come in two flavors:
/// - **Phasic** (Pacinian corpuscles, hair cells): respond to changes,
///   gate out weak sustained signals. Default for higher-level brains.
/// - **Tonic** (Merkel cells, Ruffini endings, muscle spindles): faithfully
///   transmit sustained levels. Essential for body-wall mechanoreceptors,
///   proprioception, and any interface that needs constant signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ReceptorMode {
    /// Phasic: emphasizes changes, sensitivity threshold gates weak signals.
    #[default]
    Phasic,
    /// Tonic: faithfully transmits sustained levels, bypasses sensitivity threshold.
    Tonic,
}

/// Biological fiber tract types — labeled lines.
///
/// The channel IS the label. A signal on a nociceptive tract is pain
/// by definition. The brain doesn't decode — the tract determines qualia.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum FiberTractKind {
    // === Afferent (Sensory) — body → brain, carry i32 ===

    /// Proprioceptive (Ia/Ib analog): position, extension, force feedback.
    /// Very fast, thick fibers. The body's GPS.
    /// Sign: positive = extending/increasing, negative = contracting/decreasing.
    Proprioceptive = 0,

    /// Mechanoreceptive (Aβ analog): touch, pressure, texture, contact.
    /// Fast fibers. "I'm touching something."
    /// Sign: positive = contact/pressure, negative = release/withdrawal.
    Mechanoreceptive = 1,

    /// Nociceptive fast (Aδ analog): sharp pain, temperature extremes.
    /// Medium speed, myelinated. The "ouch!" signal.
    /// Sign: positive = pain onset/increasing, negative = pain subsiding.
    NociceptiveFast = 2,

    /// Nociceptive slow (C-fiber analog): burning, aching, itch.
    /// Slow, unmyelinated. The lingering hurt.
    NociceptiveSlow = 3,

    /// Interoceptive (visceral C-fiber analog): fatigue, metabolic state,
    /// exertion cost. The body's internal weather report.
    /// Sign: positive = increasing demand/distress, negative = recovery/ease.
    Interoceptive = 4,

    // === Efferent (Motor) — brain → body, carry Signal ===

    /// Motor skeletal (Aα motor analog): voluntary movement commands.
    /// Very fast, thick. "Move this limb."
    /// Polarity: +1 = activate/contract, -1 = inhibit/release.
    MotorSkeletal = 5,

    /// Motor spindle (Aγ analog): muscle tone, reflex arcs.
    /// Fast. Background postural control.
    /// Polarity: +1 = increase tone, -1 = decrease tone.
    MotorSpindle = 6,
}

impl FiberTractKind {
    pub const COUNT: usize = 7;

    /// Is this an afferent (sensory, body→brain) tract?
    #[inline]
    pub fn is_afferent(self) -> bool {
        (self as u8) < 5
    }

    /// Is this an efferent (motor, brain→body) tract?
    #[inline]
    pub fn is_efferent(self) -> bool {
        (self as u8) >= 5
    }

    /// Biological transmission speed class (0-255).
    /// Higher = faster propagation. Affects signal delay.
    pub fn base_speed(self) -> u8 {
        match self {
            Self::Proprioceptive  => 240, // Aα: fastest
            Self::Mechanoreceptive => 200, // Aβ: fast
            Self::NociceptiveFast  => 140, // Aδ: medium
            Self::NociceptiveSlow  => 40,  // C: slow
            Self::Interoceptive    => 30,  // C visceral: slowest
            Self::MotorSkeletal    => 240, // Aα motor: fastest
            Self::MotorSpindle     => 180, // Aγ: fast
        }
    }

    /// Construct from u8 ordinal. Returns None for out-of-range.
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Proprioceptive),
            1 => Some(Self::Mechanoreceptive),
            2 => Some(Self::NociceptiveFast),
            3 => Some(Self::NociceptiveSlow),
            4 => Some(Self::Interoceptive),
            5 => Some(Self::MotorSkeletal),
            6 => Some(Self::MotorSpindle),
            _ => None,
        }
    }

    /// Short biological name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Proprioceptive   => "Ia/Ib",
            Self::Mechanoreceptive => "Aβ",
            Self::NociceptiveFast  => "Aδ",
            Self::NociceptiveSlow  => "C-noci",
            Self::Interoceptive    => "C-visc",
            Self::MotorSkeletal    => "Aα-mot",
            Self::MotorSpindle     => "Aγ-spin",
        }
    }
}

/// A single fiber tract within a bundle.
///
/// Carries either motor Signal (efferent) or sensory i32 (afferent).
/// Physical properties (all u8) shape transmission fidelity.
/// The tract kind defines signal meaning (labeled line).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FiberTract {
    /// What kind of signal this tract carries (labeled line).
    pub kind: FiberTractKind,

    /// Tract dimension (number of channels).
    pub dim: usize,

    /// Motor signals: Signal vector (efferent tracts only).
    pub motor_signals: Vec<Signal>,

    /// Sensory signals: i32 vector, Weber-quantized (afferent tracts only).
    pub sensory_signals: Vec<i32>,

    /// Previous output for elasticity smoothing (motor).
    motor_prev: Vec<Signal>,

    /// Previous output for elasticity smoothing (sensory).
    sensory_prev: Vec<i32>,

    // === Physical Properties (all u8) ===

    /// Conductivity: myelination quality (0=severed, 255=perfect).
    pub conductivity: u8,

    /// Jitter: transmission noise (0=clean, 255=overwhelmed).
    pub jitter: u8,

    /// Fatigue: current exhaustion (0=fresh, 255=spent).
    pub fatigue: u8,

    /// Endurance: fatigue resistance (0=fragile, 255=tireless).
    pub endurance: u8,

    /// Elasticity: signal tracking speed (0=sluggish, 255=instant).
    pub elasticity: u8,

    /// Sensitivity: detection threshold (0=numb, 255=hypersensitive).
    /// Sensory: signals below (255 - sensitivity) are zeroed.
    /// Motor: recruitment threshold (size principle).
    pub sensitivity: u8,

    /// Gain: amplification/attenuation (128=unity).
    /// >128: amplify (motor — guitar amp). <128: attenuate (sensory — protect brain).
    pub gain: u8,

    /// Strength: max force output (motor) or acuity (sensory).
    pub strength: u8,

    /// Receptor mode: phasic (change-detecting) or tonic (level-tracking).
    /// Tonic mode bypasses the sensitivity threshold, allowing sustained
    /// signals to pass through faithfully.
    pub receptor_mode: ReceptorMode,

    // === Adaptation counters ===

    /// Lifetime activations (total ticks where tract was active).
    pub lifetime_activations: u64,

    /// Recent activation density (rolling, u8 scaled).
    /// 0=idle, 255=constant use. Drives adaptation decisions.
    pub recent_density: u8,
}

impl FiberTract {
    /// Create a new motor (efferent) tract.
    pub fn new_motor(kind: FiberTractKind, dim: usize) -> Self {
        debug_assert!(kind.is_efferent(), "new_motor called with afferent kind");
        Self {
            kind,
            dim,
            motor_signals: vec![Signal::default(); dim],
            sensory_signals: Vec::new(),
            motor_prev: vec![Signal::default(); dim],
            sensory_prev: Vec::new(),
            conductivity: 128,
            jitter: 128,
            fatigue: 0,
            endurance: 128,
            elasticity: 128,
            sensitivity: 128,
            gain: 160, // motor default: amplify
            strength: 128,
            receptor_mode: ReceptorMode::Phasic,
            lifetime_activations: 0,
            recent_density: 0,
        }
    }

    /// Create a new sensory (afferent) tract.
    pub fn new_sensory(kind: FiberTractKind, dim: usize) -> Self {
        debug_assert!(kind.is_afferent(), "new_sensory called with efferent kind");
        Self {
            kind,
            dim,
            motor_signals: Vec::new(),
            sensory_signals: vec![0i32; dim],
            motor_prev: Vec::new(),
            sensory_prev: vec![0i32; dim],
            conductivity: 128,
            jitter: 128,
            fatigue: 0,
            endurance: 128,
            elasticity: 128,
            sensitivity: 128,
            gain: 100, // sensory default: attenuate
            strength: 128,
            receptor_mode: ReceptorMode::Phasic,
            lifetime_activations: 0,
            recent_density: 0,
        }
    }

    /// Transmit motor Signal through this tract.
    ///
    /// Applies: gain → conductivity loss → fatigue → jitter → recruitment → elasticity.
    /// All u8 integer arithmetic. No floats.
    ///
    /// Input: brain cortical Signal vector.
    /// Output: shaped Signal vector written to `self.motor_signals`.
    pub fn transmit_motor(&mut self, input: &[Signal], rng_seed: u64) {
        debug_assert!(self.kind.is_efferent());
        let len = input.len().min(self.dim);
        let mut seed = rng_seed;

        for i in 0..len {
            let sig = input[i];

            // Skip zero signals
            if sig.polarity == 0 || sig.magnitude == 0 {
                // Elasticity: smooth toward zero
                let prev = &self.motor_prev[i];
                let smoothed_mag = (prev.magnitude as u16 * (255 - self.elasticity) as u16 / 255) as u8;
                self.motor_signals[i] = if smoothed_mag == 0 {
                    Signal::default()
                } else {
                    Signal { polarity: prev.polarity, magnitude: smoothed_mag }
                };
                self.motor_prev[i] = self.motor_signals[i];
                continue;
            }

            let mut mag = sig.magnitude as u32;

            // 1. Gain modulation: mag * gain / 128
            mag = mag * self.gain as u32 / 128;

            // 2. Conductivity loss: mag * conductivity / 255
            mag = mag * self.conductivity as u32 / 255;

            // 3. Fatigue degradation: mag * (255 - fatigue) / 255
            mag = mag * (255u32.saturating_sub(self.fatigue as u32)) / 255;

            // 4. Jitter noise
            let mut polarity = sig.polarity;
            if self.jitter > 0 {
                seed = xorshift(seed);
                let noise = (seed % (self.jitter as u64 / 4 + 1)) as i32;
                seed = xorshift(seed);
                let sign = if seed % 2 == 0 { 1i32 } else { -1i32 };
                mag = (mag as i32 + noise * sign).clamp(0, 512) as u32;

                // Severe jitter: chance of polarity flip
                if self.jitter > 200 {
                    seed = xorshift(seed);
                    if seed % 8 == 0 {
                        polarity = -polarity;
                    }
                }
            }

            // 5. Recruitment threshold (size principle)
            let threshold = 255u32.saturating_sub(self.sensitivity as u32);
            if mag < threshold {
                mag = 0;
            }

            // Clamp to u8
            let target_mag = mag.min(255) as u8;

            // 6. Elasticity smoothing (signed to handle decreasing signals)
            let prev_mag = self.motor_prev[i].magnitude as i32;
            let delta = target_mag as i32 - prev_mag;
            let smoothed = prev_mag + delta * self.elasticity as i32 / 255;
            let final_mag = smoothed.clamp(0, 255) as u8;

            let out = if final_mag == 0 {
                Signal::default()
            } else {
                Signal { polarity, magnitude: final_mag }
            };

            self.motor_signals[i] = out;
            self.motor_prev[i] = out;
        }

        // Zero remaining channels
        for i in len..self.dim {
            self.motor_signals[i] = Signal::default();
            self.motor_prev[i] = Signal::default();
        }
    }

    /// Transmit sensory i32 through this tract.
    ///
    /// Input: raw environmental stimulus (i32 vector).
    /// Applies: Weber quantize → gain → conductivity → fatigue → jitter → threshold → elasticity.
    /// Output: shaped i32 vector written to `self.sensory_signals`.
    /// Brain receives these directly into HotBuffer.
    pub fn transmit_sensory(&mut self, input: &[i32], rng_seed: u64) {
        debug_assert!(self.kind.is_afferent());
        let len = input.len().min(self.dim);
        let mut seed = rng_seed;

        for i in 0..len {
            // Weber-law quantization
            let quantized = weber_quantize(input[i]);

            // Skip zero
            if quantized == 0 {
                let prev = self.sensory_prev[i];
                let smoothed = prev * (255 - self.elasticity as i32) / 255;
                self.sensory_signals[i] = smoothed;
                self.sensory_prev[i] = smoothed;
                continue;
            }

            let mut val = quantized as i64;

            // 1. Gain modulation: val * gain / 128
            val = val * self.gain as i64 / 128;

            // 2. Conductivity loss: val * conductivity / 255
            val = val * self.conductivity as i64 / 255;

            // 3. Fatigue degradation: val * (255 - fatigue) / 255
            val = val * (255i64 - self.fatigue as i64) / 255;

            // 4. Jitter noise
            if self.jitter > 0 {
                seed = xorshift(seed);
                let noise_range = self.jitter as i64;
                let noise = (seed as i64 % (noise_range + 1)) - noise_range / 2;
                val += noise;
            }

            // 5. Sensitivity threshold (scaled for i32 range)
            // Tonic receptors bypass threshold — they report absolute levels.
            // Phasic receptors gate weak signals to emphasize changes.
            if self.receptor_mode == ReceptorMode::Phasic {
                let threshold = (255i64 - self.sensitivity as i64) * 4;
                if val.abs() < threshold {
                    val = 0;
                }
            }

            // Clamp to i32 range
            let target = val.clamp(i32::MIN as i64, i32::MAX as i64) as i32;

            // 6. Elasticity smoothing
            let prev = self.sensory_prev[i] as i64;
            let smoothed = prev + (target as i64 - prev) * self.elasticity as i64 / 255;
            let final_val = smoothed.clamp(i32::MIN as i64, i32::MAX as i64) as i32;

            self.sensory_signals[i] = final_val;
            self.sensory_prev[i] = final_val;
        }

        // Zero remaining channels
        for i in len..self.dim {
            self.sensory_signals[i] = 0;
            self.sensory_prev[i] = 0;
        }
    }

    /// Whether any signal is actively being transmitted.
    pub fn is_active(&self) -> bool {
        if self.kind.is_efferent() {
            self.motor_signals.iter().any(|s| s.polarity != 0 && s.magnitude > 0)
        } else {
            self.sensory_signals.iter().any(|&v| v != 0)
        }
    }

    /// Sum of absolute signal magnitudes (activity level).
    pub fn activity_level(&self) -> u64 {
        if self.kind.is_efferent() {
            self.motor_signals.iter().map(|s| s.magnitude as u64).sum()
        } else {
            self.sensory_signals.iter().map(|v| v.unsigned_abs() as u64).sum()
        }
    }
}

/// Fast deterministic PRNG for jitter noise.
#[inline]
fn xorshift(mut state: u64) -> u64 {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn motor_tract_amplifies() {
        let mut tract = FiberTract::new_motor(FiberTractKind::MotorSkeletal, 4);
        tract.gain = 200; // amplify
        tract.conductivity = 255; // perfect
        tract.jitter = 0; // no noise
        tract.sensitivity = 255; // no threshold
        tract.elasticity = 255; // instant tracking

        let input = [
            Signal { polarity: 1, magnitude: 100 },
            Signal { polarity: -1, magnitude: 50 },
            Signal::default(),
            Signal { polarity: 1, magnitude: 10 },
        ];

        tract.transmit_motor(&input, 42);

        // 100 * 200/128 = 156
        assert_eq!(tract.motor_signals[0].polarity, 1);
        assert_eq!(tract.motor_signals[0].magnitude, 156);

        // 50 * 200/128 = 78
        assert_eq!(tract.motor_signals[1].polarity, -1);
        assert_eq!(tract.motor_signals[1].magnitude, 78);

        // Zero in → zero out
        assert_eq!(tract.motor_signals[2].magnitude, 0);

        // 10 * 200/128 = 15
        assert_eq!(tract.motor_signals[3].polarity, 1);
        assert_eq!(tract.motor_signals[3].magnitude, 15);
    }

    #[test]
    fn motor_tract_fatigue_degrades() {
        let mut tract = FiberTract::new_motor(FiberTractKind::MotorSkeletal, 1);
        tract.gain = 128; // unity
        tract.conductivity = 255;
        tract.jitter = 0;
        tract.sensitivity = 255;
        tract.elasticity = 255;
        tract.fatigue = 128; // 50% fatigued

        let input = [Signal { polarity: 1, magnitude: 200 }];
        tract.transmit_motor(&input, 0);

        // 200 * 128/128 * 255/255 * (255-128)/255 = 200 * 127/255 ≈ 99
        assert_eq!(tract.motor_signals[0].magnitude, 99);
    }

    #[test]
    fn sensory_tract_weber_quantizes() {
        let mut tract = FiberTract::new_sensory(FiberTractKind::Proprioceptive, 4);
        tract.gain = 128; // unity
        tract.conductivity = 255;
        tract.jitter = 0;
        tract.sensitivity = 255;
        tract.elasticity = 255;

        let input = [23, 130, 500, -47];
        tract.transmit_sensory(&input, 0);

        // 23 → weber step 5 → 20, then unity gain/cond → 20
        assert_eq!(tract.sensory_signals[0], 20);
        // 130 → weber step 10 → 130, unity → 130
        assert_eq!(tract.sensory_signals[1], 130);
        // 500 → weber step 15 → 495, unity → 495
        assert_eq!(tract.sensory_signals[2], 495);
        // -47 → weber step 5 → -45, unity → -45
        assert_eq!(tract.sensory_signals[3], -45);
    }

    #[test]
    fn sensory_threshold_gates() {
        let mut tract = FiberTract::new_sensory(FiberTractKind::Mechanoreceptive, 2);
        tract.gain = 128;
        tract.conductivity = 255;
        tract.jitter = 0;
        tract.sensitivity = 128; // threshold = (255-128)*4 = 508
        tract.elasticity = 255;

        let input = [100, 600];
        tract.transmit_sensory(&input, 0);

        // 100 → weber → 100, below threshold 508 → 0
        assert_eq!(tract.sensory_signals[0], 0);
        // 600 → weber step 15 → 600, above threshold → passes
        assert!(tract.sensory_signals[1] != 0);
    }

    #[test]
    fn tonic_receptor_bypasses_threshold() {
        let mut tract = FiberTract::new_sensory(FiberTractKind::Mechanoreceptive, 2);
        tract.gain = 100;        // attenuate (same as worm default)
        tract.conductivity = 128;
        tract.jitter = 0;
        tract.sensitivity = 180; // phasic threshold = (255-180)*4 = 300
        tract.elasticity = 255;
        tract.receptor_mode = ReceptorMode::Tonic;

        // Raw 500 → Weber 495 → gain 386 → cond 193
        // Phasic: 193 < 300 threshold → ZERO
        // Tonic: threshold bypassed → 193 passes
        let input = [500, 100];
        tract.transmit_sensory(&input, 0);

        assert!(tract.sensory_signals[0] != 0, "tonic should pass: {}", tract.sensory_signals[0]);
        assert!(tract.sensory_signals[1] != 0, "tonic should pass weak too: {}", tract.sensory_signals[1]);
    }

    #[test]
    fn phasic_still_gates_by_default() {
        let mut tract = FiberTract::new_sensory(FiberTractKind::Mechanoreceptive, 1);
        tract.gain = 100;
        tract.conductivity = 128;
        tract.jitter = 0;
        tract.sensitivity = 180;
        tract.elasticity = 255;
        // receptor_mode defaults to Phasic

        let input = [500];
        tract.transmit_sensory(&input, 0);

        // 500 → 495 → gain 386 → cond 193 → threshold 300 → ZERO
        assert_eq!(tract.sensory_signals[0], 0, "phasic should gate this");
    }

    #[test]
    fn tract_kind_classification() {
        assert!(FiberTractKind::Proprioceptive.is_afferent());
        assert!(FiberTractKind::NociceptiveFast.is_afferent());
        assert!(FiberTractKind::Interoceptive.is_afferent());
        assert!(FiberTractKind::MotorSkeletal.is_efferent());
        assert!(FiberTractKind::MotorSpindle.is_efferent());
        assert!(!FiberTractKind::Proprioceptive.is_efferent());
        assert!(!FiberTractKind::MotorSkeletal.is_afferent());
    }
}
