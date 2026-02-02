# fibertract

Peripheral nervous system substrate for neuromorphic systems.

Twin primitive to [neuropool](https://github.com/blackfall-labs/neuropool-rs). Where neuropool provides spiking neural substrate (brain tissue), fibertract provides connective tissue (peripheral nerves).

## Core Concepts

**Fiber tracts are labeled lines.** The tract kind defines what signals on that tract *mean*. A signal on a nociceptive tract IS pain. A signal on a proprioceptive tract IS body position. Same data format, different qualia -- determined by which tract carries it.

**Asymmetric encoding.** Motor tracts (efferent, brain -> body) carry `Signal` -- the brain's native ternary format `{polarity: i8, magnitude: u8}`. Sensory tracts (afferent, body -> brain) carry `i32` -- wide dynamic range with Weber-law quantization for biologically realistic discrimination.

**Use-dependent adaptation.** Tracts don't learn (no plasticity). They adapt: conductivity improves with use, jitter decreases with practice, strength grows with sustained effort, endurance builds over time. Idle tracts atrophy. This is connective tissue, not neural substrate.

## Architecture

```
FiberBundle ("left_hand", "gaze", "vocal_tract", ...)
  +-- Vec<FiberTract>
        |-- MotorSkeletal   [Signal]  brain -> body (amplify)
        |-- Proprioceptive  [i32]     body -> brain (position)
        |-- NociceptiveFast [i32]     body -> brain (sharp pain)
        |-- Interoceptive   [i32]     body -> brain (fatigue)
        +-- ...
```

## Fiber Types

| Kind | Direction | Speed | Biological Analog | Carries |
|------|-----------|-------|-------------------|---------|
| Proprioceptive | Afferent | Ia/Ib (fastest) | Position, extension, force | `i32` |
| Mechanoreceptive | Afferent | A-beta (fast) | Touch, pressure, texture | `i32` |
| NociceptiveFast | Afferent | A-delta (medium) | Sharp pain, temperature | `i32` |
| NociceptiveSlow | Afferent | C-fiber (slow) | Burning, aching, itch | `i32` |
| Interoceptive | Afferent | C-visceral (slowest) | Fatigue, metabolic state | `i32` |
| MotorSkeletal | Efferent | A-alpha (fastest) | Voluntary movement | `Signal` |
| MotorSpindle | Efferent | A-gamma (fast) | Muscle tone, reflexes | `Signal` |

## Physical Properties (all u8, no floats)

| Property | Range | Meaning |
|----------|-------|---------|
| Conductivity | 0=severed, 255=perfect | Myelination quality |
| Jitter | 0=clean, 255=overwhelmed | Transmission noise |
| Gain | 128=unity, >128=amplify | Signal amplification |
| Sensitivity | 0=numb, 255=hypersensitive | Detection threshold |
| Fatigue | 0=fresh, 255=spent | Current exhaustion |
| Endurance | 0=fragile, 255=tireless | Fatigue resistance |
| Strength | 0=atrophied, 255=peak | Max force / acuity |
| Elasticity | 0=sluggish, 255=instant | Signal tracking speed |

## Weber-Law Quantization

Sensory signals are quantized using Weber's law -- just-noticeable difference is proportional to stimulus magnitude:

| Magnitude | Step Size | Example |
|-----------|-----------|---------|
| 0-49 | 5 | You notice 1g vs 6g |
| 50-199 | 10 | You notice 100g vs 110g |
| 200-999 | 15 | You barely notice 500g vs 515g |
| 1000+ | 25 | You can't tell 2000g from 2020g |

## Transmission Pipeline

Motor (brain -> body):
```
Signal -> Gain -> Conductivity loss -> Fatigue -> Jitter -> Recruitment -> Elasticity -> Output
```

Sensory (body -> brain):
```
Raw i32 -> Weber quantize -> Gain -> Conductivity -> Fatigue -> Jitter -> Sensitivity -> Elasticity -> Output
```

## Body Profiles

Pre-configured `LimbProfile` presets for biologically appropriate fiber bundles:

| Profile | Motor Dim | Key Traits |
|---------|-----------|------------|
| `hand` | 32 | High dexterity, dense touch (64ch), low jitter |
| `arm` | 16 | Good strength, balanced sensory |
| `leg` | 12 | High strength/endurance, coarse motor, high proprioception |
| `vocal_tract` | 24 | Extreme precision, near-instant elasticity, minimal pain |
| `gaze` | 12 | Fastest response, cleanest signals, no pain fibers |
| `torso` | 8 | Core stability, high visceral awareness |

## Usage

```rust
use fibertract::{FiberBundle, FiberTract, FiberTractKind, LimbProfile};

// Build a hand from preset profile
let profile = LimbProfile::hand("left");
let mut hand = profile.build();

// Chemical modulation (affects entire bundle)
hand.apply_adrenaline(200);  // fight-or-flight: boost motor, gate pain

// Adapt over time (call each tick)
use fibertract::adapt::{adapt_bundle, AdaptationConfig};
let config = AdaptationConfig::default();
adapt_bundle(&mut hand, &config);
```

## Pain System

Pain events carry perceived intensity (after chemical gating), onset sharpness, duration, and habituation state. The brain uses salience scoring to allocate attention:

```rust
use fibertract::{PainEvent, PainSource};

let event = PainEvent {
    bundle_name: "left_hand".into(),
    source: PainSource::Sharp,
    intensity: 200,
    onset: 220,
    duration_ticks: 5,
    habituating: false,
};

assert!(event.is_urgent());      // high intensity + high urgency source
assert_eq!(event.salience(), _);  // weighted score for attention
```

## Features

- `serde` (default) -- enables serialization for all types via serde + serde_json

## Dependencies

- [ternary-signal](https://crates.io/crates/ternary-signal) -- the `Signal` type ({polarity, magnitude})
- `log` -- structured logging
- `rand` -- jitter noise generation

## License

MIT OR Apache-2.0
