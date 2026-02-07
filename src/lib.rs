//! Fibertract — peripheral nervous system substrate.
//!
//! Twin primitive to [neuropool](https://crates.io/crates/neuropool).
//! Where neuropool provides spiking neural substrate (brain tissue),
//! fibertract provides connective tissue (peripheral nerves).
//!
//! # Core Concepts
//!
//! **Fiber tracts are labeled lines.** The tract kind defines what signals
//! on that tract MEAN. A signal on a nociceptive tract IS pain. A signal
//! on a proprioceptive tract IS body position. Same data format, different
//! qualia — determined by which tract carries it.
//!
//! **Asymmetric encoding.** Motor tracts (efferent, brain→body) carry
//! [`Signal`] — the brain's native ternary format `{polarity: i8, magnitude: u8}`.
//! Sensory tracts (afferent, body→brain) carry `i32` — wide dynamic range
//! with Weber-law quantization for biologically realistic discrimination.
//!
//! **Use-dependent adaptation.** Tracts don't learn (no plasticity). They
//! adapt: conductivity improves with use, jitter decreases with practice,
//! strength grows with sustained effort, endurance builds over time.
//! Idle tracts atrophy. This is connective tissue, not neural substrate.
//!
//! # Architecture
//!
//! ```text
//! FiberBundle ("locomotion", "gaze", "vocalization", ...)
//!   └── Vec<FiberTract>
//!         ├── MotorSkeletal   [Signal]  brain → body (amplify)
//!         ├── Proprioceptive  [i32]     body → brain (position)
//!         ├── NociceptiveFast [i32]     body → brain (sharp pain)
//!         ├── Interoceptive   [i32]     body → brain (fatigue)
//!         └── ...
//! ```
//!
//! # Properties (all u8, no floats)
//!
//! - **Conductivity**: myelination quality. 0=severed, 255=perfect.
//! - **Jitter**: transmission noise. 0=clean, 255=overwhelmed.
//! - **Gain**: amplification. 128=unity, >128=amplify (motor), <128=attenuate (sensory).
//! - **Sensitivity**: detection threshold. 0=numb, 255=hypersensitive.
//! - **Fatigue**: current exhaustion. 0=fresh, 255=spent.
//! - **Endurance**: fatigue resistance. 0=fragile, 255=tireless.
//! - **Strength**: max force (motor) or acuity (sensory). 0=atrophied, 255=peak.
//! - **Elasticity**: signal tracking speed. 0=sluggish, 255=instant.

pub mod tract;
pub mod bundle;
pub mod pain;
pub mod adapt;
pub mod weber;
pub mod profile;

pub use tract::{FiberTract, FiberTractKind, ReceptorMode};
pub use bundle::FiberBundle;
pub use pain::{PainEvent, PainSource};
pub use profile::LimbProfile;
pub use weber::weber_quantize;
