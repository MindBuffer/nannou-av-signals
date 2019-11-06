extern crate nannou;

use nannou::ease::*;
use nannou::math::fmod;
use nannou::prelude::*;
use nannou::rand::random_f32;

pub const ALL: &'static [Signal] = &[
    Signal::SINE,
    Signal::TRIANGLE,
    Signal::SAWTOOTH,
    Signal::SQUARE,
    Signal::NOISE,
    Signal::BACK_IN,
    Signal::BACK_IN_OUT,
    Signal::BACK_IN_OUT_S,
    Signal::BACK_IN_S,
    Signal::BACK_OUT,
    Signal::BACK_OUT_S,
    Signal::BOUNCE_IN,
    Signal::BOUNCE_IN_OUT,
    Signal::BOUNCE_OUT,
    Signal::CIRC_IN,
    Signal::CIRC_IN_OUT,
    Signal::CIRC_OUT,
    Signal::CUBIC_IN,
    Signal::CUBIC_IN_OUT,
    Signal::CUBIC_OUT,
    Signal::ELASTIC_IN,
    Signal::ELASTIC_IN_OUT,
    Signal::ELASTIC_OUT,
    Signal::EXPO_IN,
    Signal::EXPO_IN_OUT,
    Signal::EXPO_OUT,
    Signal::QUAD_IN,
    Signal::QUAD_IN_OUT,
    Signal::QUAD_OUT,
    Signal::QUART_IN,
    Signal::QUART_IN_OUT,
    Signal::QUART_OUT,
    Signal::QUINT_IN,
    Signal::QUINT_IN_OUT,
    Signal::QUINT_OUT,
    Signal::SINE_IN,
    Signal::SINE_IN_OUT,
    Signal::SINE_OUT,
];

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Signal {
    Lfo(LfoType),
    Ease(EasingType),
}

impl Signal {
    pub const SINE: Self = Signal::Lfo(LfoType::Sine);
    pub const TRIANGLE: Self = Signal::Lfo(LfoType::Triangle);
    pub const SAWTOOTH: Self = Signal::Lfo(LfoType::Sawtooth);
    pub const SQUARE: Self = Signal::Lfo(LfoType::Square);
    pub const NOISE: Self = Signal::Lfo(LfoType::Noise);

    pub const BACK_IN: Self = Signal::Ease(EasingType::BackIn);
    pub const BACK_IN_OUT: Self = Signal::Ease(EasingType::BackInOut);
    pub const BACK_IN_OUT_S: Self = Signal::Ease(EasingType::BackInOutS);
    pub const BACK_IN_S: Self = Signal::Ease(EasingType::BackInS);
    pub const BACK_OUT: Self = Signal::Ease(EasingType::BackOut);
    pub const BACK_OUT_S: Self = Signal::Ease(EasingType::BackOutS);
    pub const BOUNCE_IN: Self = Signal::Ease(EasingType::BounceIn);
    pub const BOUNCE_IN_OUT: Self = Signal::Ease(EasingType::BounceInOut);
    pub const BOUNCE_OUT: Self = Signal::Ease(EasingType::BounceOut);
    pub const CIRC_IN: Self = Signal::Ease(EasingType::CircIn);
    pub const CIRC_IN_OUT: Self = Signal::Ease(EasingType::CircInOut);
    pub const CIRC_OUT: Self = Signal::Ease(EasingType::CircOut);
    pub const CUBIC_IN: Self = Signal::Ease(EasingType::CubicIn);
    pub const CUBIC_IN_OUT: Self = Signal::Ease(EasingType::CubicInOut);
    pub const CUBIC_OUT: Self = Signal::Ease(EasingType::CubicOut);
    pub const ELASTIC_IN: Self = Signal::Ease(EasingType::ElasticIn);
    pub const ELASTIC_IN_OUT: Self = Signal::Ease(EasingType::ElasticInOut);
    pub const ELASTIC_OUT: Self = Signal::Ease(EasingType::ElasticOut);
    pub const EXPO_IN: Self = Signal::Ease(EasingType::ExpoIn);
    pub const EXPO_IN_OUT: Self = Signal::Ease(EasingType::ExpoInOut);
    pub const EXPO_OUT: Self = Signal::Ease(EasingType::ExpoOut);
    pub const QUAD_IN: Self = Signal::Ease(EasingType::QuadIn);
    pub const QUAD_IN_OUT: Self = Signal::Ease(EasingType::QuadInOut);
    pub const QUAD_OUT: Self = Signal::Ease(EasingType::QuadOut);
    pub const QUART_IN: Self = Signal::Ease(EasingType::QuadIn);
    pub const QUART_IN_OUT: Self = Signal::Ease(EasingType::QuartInOut);
    pub const QUART_OUT: Self = Signal::Ease(EasingType::QuartOut);
    pub const QUINT_IN: Self = Signal::Ease(EasingType::QuintIn);
    pub const QUINT_IN_OUT: Self = Signal::Ease(EasingType::QuintInOut);
    pub const QUINT_OUT: Self = Signal::Ease(EasingType::QuintOut);
    pub const SINE_IN: Self = Signal::Ease(EasingType::SineIn);
    pub const SINE_IN_OUT: Self = Signal::Ease(EasingType::SineInOut);
    pub const SINE_OUT: Self = Signal::Ease(EasingType::SineOut);

    pub fn amp(&self, phase: f32) -> f32 {
        match self {
            Signal::Lfo(lfo_type) => lfo_type.amp(phase),
            Signal::Ease(ease_type) => ease_type.amp(phase),
        }
    }

    pub fn all_names() -> Vec<String> {
        let mut list = Vec::new();
        for signal in ALL {
            list.push(format!("{:?}", signal));
        }
        list
    }
}

//------------------ LFO'S
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LfoType {
    Sine,
    Triangle,
    Sawtooth,
    Square,
    Noise,
}

impl LfoType {
    pub fn amp(&self, phase: f32) -> f32 {
        lfo(*self, phase)
    }
}

pub fn lfo(lfo_type: LfoType, phase: f32) -> f32 {
    match lfo_type {
        LfoType::Sine => sine(phase),
        LfoType::Triangle => triangle(phase),
        LfoType::Sawtooth => sawtooth(phase),
        LfoType::Square => square(phase),
        LfoType::Noise => noise(phase),
    }
}

fn sine(phase: f32) -> f32 {
    ((PI * 2.0) * phase).sin()
}
fn triangle(phase: f32) -> f32 {
    (phase * -2.0 + 1.0).abs() * 2.0 - 1.0
}
fn square(phase: f32) -> f32 {
    (if fmod(phase, 1.0) < 0.5 { -1.0 } else { 1.0 })
}
fn sawtooth(phase: f32) -> f32 {
    fmod(phase, 1.0) * -2.0 + 1.0
}
fn noise(phase: f32) -> f32 {
    random_f32() * 2.0 - 1.0
}

//------------------ EASINGS
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EasingType {
    BackIn,
    BackInOut,
    BackInOutS,
    BackInS,
    BackOut,
    BackOutS,
    BounceIn,
    BounceInOut,
    BounceOut,
    CircIn,
    CircInOut,
    CircOut,
    CubicIn,
    CubicInOut,
    CubicOut,
    ElasticIn,
    ElasticInOut,
    ElasticOut,
    ExpoIn,
    ExpoInOut,
    ExpoOut,
    QuadIn,
    QuadInOut,
    QuadOut,
    QuartIn,
    QuartInOut,
    QuartOut,
    QuintIn,
    QuintInOut,
    QuintOut,
    SineIn,
    SineInOut,
    SineOut,
}

impl EasingType {
    pub fn amp(&self, phase: f32) -> f32 {
        ease_lfo(*self, fmod(phase, 1.0)) * 2.0 - 1.0
    }
}

pub fn ease_lfo(ease_type: EasingType, phase: f32) -> f32 {
    let from = 0.0;
    let distance = 1.0;
    let duration = 1.0;

    match ease_type {
        // Back
        // the back easing equation can receive an extra parameter in the _s functions
        // that controls how much the easing goes forward or backwards
        EasingType::BackIn => back::ease_in(phase, from, distance, duration),
        EasingType::BackInOut => back::ease_in_out(phase, from, distance, duration),
        EasingType::BackInOutS => back::ease_in_out_s(phase, from, distance, duration, 0.8),
        EasingType::BackInS => back::ease_in_s(phase, from, distance, duration, 0.8),
        EasingType::BackOut => back::ease_out(phase, from, distance, duration),
        EasingType::BackOutS => back::ease_out_s(phase, from, distance, duration, 0.8),
        // Bounce
        EasingType::BounceIn => bounce::ease_in(phase, from, distance, duration),
        EasingType::BounceInOut => bounce::ease_in_out(phase, from, distance, duration),
        EasingType::BounceOut => bounce::ease_out(phase, from, distance, duration),
        // Circ
        EasingType::CircIn => circ::ease_in(phase, from, distance, duration),
        EasingType::CircInOut => circ::ease_in_out(phase, from, distance, duration),
        EasingType::CircOut => circ::ease_out(phase, from, distance, duration),
        // Cubic
        EasingType::CubicIn => cubic::ease_in(phase, from, distance, duration),
        EasingType::CubicInOut => cubic::ease_in_out(phase, from, distance, duration),
        EasingType::CubicOut => cubic::ease_out(phase, from, distance, duration),
        // Elastic
        EasingType::ElasticIn => elastic::ease_in(phase, from, distance, duration),
        EasingType::ElasticInOut => elastic::ease_in_out(phase, from, distance, duration),
        EasingType::ElasticOut => elastic::ease_out(phase, from, distance, duration),
        // Expo
        EasingType::ExpoIn => expo::ease_in(phase, from, distance, duration),
        EasingType::ExpoInOut => expo::ease_in_out(phase, from, distance, duration),
        EasingType::ExpoOut => expo::ease_out(phase, from, distance, duration),
        // Quad
        EasingType::QuadIn => quad::ease_in(phase, from, distance, duration),
        EasingType::QuadInOut => quad::ease_in_out(phase, from, distance, duration),
        EasingType::QuadOut => quad::ease_out(phase, from, distance, duration),
        // Quart
        EasingType::QuartIn => quart::ease_in(phase, from, distance, duration),
        EasingType::QuartInOut => quart::ease_in_out(phase, from, distance, duration),
        EasingType::QuartOut => quart::ease_in_out(phase, from, distance, duration),
        // Quint
        EasingType::QuintIn => quint::ease_in(phase, from, distance, duration),
        EasingType::QuintInOut => quint::ease_in_out(phase, from, distance, duration),
        EasingType::QuintOut => quint::ease_in_out(phase, from, distance, duration),
        // Sine
        EasingType::SineIn => sine::ease_in(phase, from, distance, duration),
        EasingType::SineInOut => sine::ease_in_out(phase, from, distance, duration),
        EasingType::SineOut => sine::ease_in_out(phase, from, distance, duration),
    }
}
