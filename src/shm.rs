// Simple Harmonic Motion module
use nannou::math::fmod;
use nannou::prelude::*;
use nannou::state::time::Duration;
use signals::{self, ease_lfo, lfo, EasingType, LfoType, Signal};

pub struct Shm {
    signal_type: Signal, // LFO or the fract component of an Easing Curve
    phases: Vec<f32>,    // Our vector of oscillator phases per SHM block
    start_angle: f32,    // The master frequency
    pub a_velocity: f32, // Defines the phase offsets between oscillators
    angle_offset: f32,   // Defines the specific offset pattern, set by offset_hz
    pub hz: f32,         // Master Speed of all osciallators
    pub offset_hz: f32,  // The rate at which oscillators fall in and out of phase
    pub skew: f32,       // Skew the waveform in a direction
    pub mirror: bool,    // Defines is our phases are mirrored or not
}

impl Shm {
    /// Construct a new shm module with an initial size
    pub fn new(size: usize, hz: f32, a_velocity: f32, offset_hz: f32) -> Self {
        let signal_type = Signal::SINE;
        let phases = vec![0.0; size];
        let start_angle = 0.0;
        let angle_offset = 0.0;
        let skew = 1.0;
        let mirror = false;
        Shm {
            signal_type,
            phases,
            hz,
            start_angle,
            angle_offset,
            a_velocity,
            offset_hz,
            skew,
            mirror,
        }
    }

    /// Defines the signal that the shm module will use, can be
    /// either an LFO or and Easing Type
    pub fn set_signal_type(&mut self, signal_type: Signal) {
        self.signal_type = signal_type;
    }

    /// Set the number or signals
    pub fn set_size(&mut self, size: usize) {
        self.phases.resize(size, 0.0);
    }

    /// Get the size of the shm vector
    pub fn size(&self) -> usize {
        self.phases.len()
    }

    /// Return an immutable reference to the underlying slice
    /// our phases vector
    pub fn phases(&self) -> &[f32] {
        &self.phases
    }

    pub fn update(&mut self) {
        self.start_angle += self.hz * 0.05;
        let mut angle = self.start_angle;

        if self.mirror {
            let half_size = self.size() as f32 / 2.0;
            let (first, last) = self.phases.split_at_mut(half_size.round() as usize);

            for p in first.iter_mut() {
                *p = self.signal_type.amp(fmod(angle, 1.0).powf(self.skew));
                angle += self.a_velocity + self.angle_offset;
                self.angle_offset += self.offset_hz * 0.00005;
            }
            for (l, f) in last.iter_mut().rev().zip(first) {
                *l = *f;
            }
        } else {
            for p in self.phases.iter_mut() {
                *p = self.signal_type.amp(fmod(angle, 1.0).powf(self.skew));
                angle += self.a_velocity + self.angle_offset;
                self.angle_offset += self.offset_hz * 0.00005;
            }
        }
    }
}
