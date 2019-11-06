mod shm;
mod signals;
mod gui;

use nannou::prelude::*;
use nannou::Ui;
use nannou_audio::{self as audio, Buffer};
use nannou_laser as laser;
use shm::Shm;
use signals::Signal;


fn main() {
    nannou::app(model).update(update).run();
}

pub struct SignalParams {
    signal_names: Vec<String>,
    selected_idx: Option<usize>,
    pow: f32,
    min: f32,
    max: f32,
    invert: bool, // Invert all of the phases
    dmx_on: bool,
    laser_on: bool,
    audio_on: bool,
}

struct Model {
    dmx: Dmx,
    audio_stream: audio::Stream<Audio>,
    laser_stream: laser::FrameStream<Laser>,
    shm: Shm,
    ui: Ui,
    ids: gui::Ids,
    params: SignalParams,
    phases: Vec<f32>,
}

struct Oscillator {
    phase: f64,
    hz: f64,
}

struct Laser {
    positions: Vec<f32>,
}

struct Audio {
    oscillators: Vec<Oscillator>,
}

struct Dmx {
    source: Option<sacn::DmxSource>,
    buffer: Vec<u8>,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .with_dimensions(1200, 750)
        .view(view)
        .build()
        .unwrap();
    
    //let mut shm = Shm::new(points_per_frame as usize, 0.1, 0.49, 0.0);
    let mut shm = Shm::new(34, 0.1, 0.49, 0.0);
    shm.set_signal_type(Signal::SINE_IN_OUT);

    let phases = vec![0.0; shm.size()];

    // Create the UI
    let mut ui = app.new_ui().build().unwrap();

    // Generate some ids for our widgets
    let ids = gui::Ids::new(ui.widget_id_generator());

    // Initialise the state that we want to live on the audio thread.
    let oscillators = (0..shm.size()).map(|_| Oscillator{phase: 0.0, hz: 100.0}).collect();
    let audio_model = Audio {
        oscillators,
    };
    
    // Initialise the audio API so we can spawn an audio stream.
    let audio_host = audio::Host::new();
    let audio_stream = audio_host
        .new_output_stream(audio_model)
        .render(audio)
        .build()
        .unwrap();

    // Initialise the state that we want to live on the laser thread and spawn the stream.
    let laser_model = Laser {
        positions: Vec::new(),
    };
    let _laser_api = laser::Api::new();
    let laser_stream = _laser_api
        .new_frame_stream(laser_model, laser)
        .build()
        .unwrap();

    let dmx = Dmx {
        source: None,
        buffer: vec![],
    };

    let params = SignalParams {
        signal_names: signals::Signal::all_names(),
        selected_idx: None,
        pow: 1.0,
        min: -1.0,
        max: 1.0,
        invert: false,
        dmx_on: true,
        laser_on: true,
        audio_on: false,
    };

    Model {
        dmx,
        audio_stream,
        laser_stream,
        shm,
        ui,
        ids,
        params,
        phases,
    }
}

fn update(_app: &App, m: &mut Model, _update: Update) {
    // Apply the GUI update.
    let ui = m.ui.set_widgets();
    gui::update(
        ui,
        &mut m.ids,
        &mut m.params,
        &mut m.shm,
    );

    m.shm.update();
    if m.params.selected_idx.is_some() {
        m.shm.set_signal_type(signals::ALL[m.params.selected_idx.unwrap()]);
    }

    m.phases = m.shm.phases.iter()
        .map(|p| {
            let mut phase = map_range(p.clone(), -1.0, 1.0, 0.0, 1.0);
            phase = phase.powf(m.params.pow);

            match m.params.invert {
                true => map_range(phase, 0.0, 1.0, m.params.max, m.params.min),
                false => map_range(phase, 0.0, 1.0, m.params.min, m.params.max),
            }        
        })
        .collect();

    // Ensure we are connected to a DMX source if enabled.
    if m.params.dmx_on && m.dmx.source.is_none() {
        let source = sacn::DmxSource::new("Nannou Signals")
            .expect("failed to connect to DMX source");
        m.dmx.source = Some(source);
    } else if !m.params.dmx_on && m.dmx.source.is_some() {
        m.dmx.source.take();
    }

    // If we have a DMX source, send data over it!
    match (&m.dmx.source, m.params.dmx_on) {
        (Some(ref dmx_source), true) => {
            m.dmx.buffer.clear();
            // Make sure we remap our normalised phase values
            // to u8 values between 0 & 255 for DMX 
            m.dmx.buffer.extend(m.phases.iter()
                .map(|p| (*p * 255.0) as u8)
            );

            // Send DMX data.
            let universe = 1;
            dmx_source
                .send(universe, &m.dmx.buffer[..])
                .expect("failed to send DMX data");
            }
        _ => ()
    }

    // Send our phase data over to the audio thead
    if m.params.audio_on {
        let phases = m.phases.clone();
        m.audio_stream.send(move |audio| {
            for (osc, phase) in audio.oscillators.iter_mut().zip(phases) {
                osc.hz = phase as f64;
            }
        }).unwrap();
    }  

    // Send our phase data over to the laser thead
    if m.params.laser_on {
        let phases = m.phases.clone();
        m.laser_stream.send(move |laser| {
            for (positions, phase) in laser.positions.iter_mut().zip(phases) {
                *positions = phase;
            }
        })
        .unwrap();
    }
}

fn view(app: &App, m: &Model, frame: &Frame) {
    let draw = app.draw();
    draw.background().rgb(0.1, 0.1, 0.1);

    let win = app.window_rect();

    //let mut laser_data = Vec::new();

    let radius = win.w() / m.shm.size() as f32;
    let height = win.h() / 2.0 - 20.0;

    m.phases.iter()
        .enumerate()
        .for_each(|(i, y)| {
            let x = (win.left() + (radius * 0.5)) + i as f32 * radius;
            let (h, s, v) = (1.0 - (i as f32 / m.shm.size() as f32), 0.75, 0.5);
            draw.line()
                .start(Point2::new(x, 0.0))
                .end(Point2::new(x, y * height))
                .hsv(h, s, v);

            draw.ellipse().x_y(x, y * height).w_h(radius, radius).hsv(h, s, v);
        });

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Draw the UI
    m.ui.draw_to_frame(app, &frame).unwrap();
}

// A function that renders the given `Audio` to the given `Buffer`, returning the result of both.
fn audio(audio: &mut Audio, buffer: &mut Buffer) {
    let sample_rate = buffer.sample_rate() as f64;
    let volume = 0.5;
    for frame in buffer.frames_mut() {
        let mut output = 0.0;
        for osc in audio.oscillators.iter_mut() {
            let sine_amp = (2.0 * PI * osc.phase as f32).sin() as f32;
            osc.phase += osc.hz / sample_rate;
            osc.phase %= sample_rate;
            output += sine_amp;
        }
        output = (output / audio.oscillators.len() as f32) * volume;
        for channel in frame {
            *channel = output;
        }
    }
}

fn laser(laser: &mut Laser, frame: &mut laser::Frame) {
    let points = laser.positions
        .iter()
        .enumerate()
        .map(|(i,y)| {
            let x = map_range(i,0,laser.positions.len(), -1.0, 1.0);
            let pos = [x,*y * 2.0 - 1.0];
            let rgb = [x + 0.5 * 0.5, 0.0, *y];
            laser::Point::new(pos, rgb)
        })
        .collect::<Vec<_>>();
    
    frame.add_lines(points);
}