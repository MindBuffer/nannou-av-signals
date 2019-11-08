mod gui;
mod shm;
mod signals;

use nannou::prelude::*;
use nannou::Ui;
use nannou_audio::{self as audio, Buffer};
use nannou_laser as laser;
use shm::Shm;
use signals::Signal;
use std::sync::{mpsc, Arc};

const PIXELS_PER_LED_STRIP: u16 = 48;
const DMX_CHANNELS_PER_LED: u16 = 3;
const ADDRS_PER_STRIP: u16 = PIXELS_PER_LED_STRIP * DMX_CHANNELS_PER_LED;
const STRIPS_PER_UNIVERSE: u16 = 3;
const LED_ADDRS_PER_UNIVERSE: u16 = ADDRS_PER_STRIP * STRIPS_PER_UNIVERSE;
const NUM_LED_STRIPS: u16 = 6;
const TOTAL_LED_PIXELS: u16 = NUM_LED_STRIPS * PIXELS_PER_LED_STRIP;

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
    audio_host: audio::Host,
    audio_stream: Option<audio::Stream<Audio>>,
    laser_api: Arc<laser::Api>,
    laser_stream: Option<laser::FrameStream<Laser>>,
    laser_dac_rx: mpsc::Receiver<laser::DetectedDac>,
    detected_laser_dac: Option<laser::DetectedDac>,
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
    let mut shm = Shm::new(TOTAL_LED_PIXELS as usize, 0.1, 0.49, 0.0);
    shm.set_signal_type(Signal::SINE_IN_OUT);

    let phases = vec![0.0; shm.size()];

    // Create the UI
    let mut ui = app.new_ui().build().unwrap();

    // Generate some ids for our widgets
    let ids = gui::Ids::new(ui.widget_id_generator());

    // Initialise the audio API so we can spawn an audio stream.
    let audio_host = audio::Host::new();
    let audio_stream = None;

    // Initialise the LASER API.
    let laser_api = Arc::new(laser::Api::new());

    // A channel for receiving newly detected ether-dream DACs.
    let (laser_dac_tx, laser_dac_rx) = mpsc::channel();

    // Spawn a thread for detecting the DACs.
    let laser_api2 = laser_api.clone();
    std::thread::spawn(move || {
        let mut detected = std::collections::HashSet::new();
        for res in laser_api2
            .detect_dacs()
            .expect("failed to start detecting DACs")
        {
            let dac = res.expect("error occurred during DAC detection");
            if detected.insert(dac.id()) {
                println!("{:#?}", dac);
                if laser_dac_tx.send(dac).is_err() {
                    break;
                }
            }
        }
    });

    let detected_laser_dac = None;
    let laser_stream = None;

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
        dmx_on: false,
        laser_on: true,
        audio_on: false,
    };

    Model {
        dmx,
        audio_host,
        audio_stream,
        laser_api,
        laser_stream,
        laser_dac_rx,
        detected_laser_dac,
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
    gui::update(ui, &mut m.ids, &mut m.params, &mut m.shm);

    // First, check for new laser DACs.
    for dac in m.laser_dac_rx.try_recv() {
        println!("Detected LASER DAC {:?}!", dac.id());
        m.detected_laser_dac = Some(dac);
    }

    // If the laser is turned on and we have a DAC, create our stream!
    match (
        m.laser_stream.is_some(),
        m.params.laser_on,
        m.detected_laser_dac.as_ref(),
    ) {
        (false, true, Some(dac)) => {
            let laser_model = Laser {
                positions: Vec::new(),
            };
            let stream = m
                .laser_api
                .new_frame_stream(laser_model, laser)
                .detected_dac(dac.clone())
                .build()
                .expect("failed to establish stream with newly detected DAC");
            m.laser_stream = Some(stream);
        }
        (true, false, _) => {
            m.laser_stream.take();
        }
        _ => (),
    }

    // Create or destroy the audio stream if necessary.
    if m.audio_stream.is_none() && m.params.audio_on {
        let oscillators = (0..m.phases.len())
            .map(|_| Oscillator {
                phase: 0.0,
                hz: 100.0,
            })
            .collect();
        let audio_model = Audio { oscillators };
        let stream = m
            .audio_host
            .new_output_stream(audio_model)
            .render(audio)
            .build()
            .unwrap();
        m.audio_stream = Some(stream);
    } else if m.audio_stream.is_some() && !m.params.audio_on {
        let stream = m.audio_stream.take().unwrap();
        stream.pause().ok();
    }

    // Ensure we are connected to a DMX source if enabled.
    if m.params.dmx_on && m.dmx.source.is_none() {
        let source =
            sacn::DmxSource::new("Nannou Signals").expect("failed to connect to DMX source");
        m.dmx.source = Some(source);
    } else if !m.params.dmx_on && m.dmx.source.is_some() {
        m.dmx.source.take();
    }

    // Update the simple harmonic motion.
    m.shm.update();
    if m.params.selected_idx.is_some() {
        m.shm
            .set_signal_type(signals::ALL[m.params.selected_idx.unwrap()]);
    }

    // Apply the invert and pow GUI controls to the SHM phases to get our actual phases.
    m.phases = m
        .shm
        .phases()
        .iter()
        .map(|p| {
            let mut phase = map_range(p.clone(), -1.0, 1.0, 0.0, 1.0);
            phase = phase.powf(m.params.pow);
            match m.params.invert {
                true => map_range(phase, 0.0, 1.0, m.params.max, m.params.min),
                false => map_range(phase, 0.0, 1.0, m.params.min, m.params.max),
            }
        })
        .collect();

    // If we have a DMX source, send data over it!
    if let (Some(dmx_source), true) = (&m.dmx.source, m.params.dmx_on) {
        m.dmx.buffer.clear();

        // Use the pixel index to determine which phase to select for our brightness values.
        for i in 0..TOTAL_LED_PIXELS {
            let phase_ix = ((i as f64 / TOTAL_LED_PIXELS as f64) * m.phases.len() as f64) as usize;
            let phase = m.phases[phase_ix];
            let byte = (phase * std::u8::MAX as f32) as u8;
            let rgb = [byte; 3];
            m.dmx.buffer.extend(rgb.iter().cloned());
        }

        let mut universe = 1;

        // If we've filled all the colour for one universe, send it.
        while !m.dmx.buffer.is_empty() {
            let data = &m.dmx.buffer[..LED_ADDRS_PER_UNIVERSE as usize];
            dmx_source
                .send(universe, data)
                .expect("failed to send LED DMX data");
            m.dmx.buffer.drain(..LED_ADDRS_PER_UNIVERSE as usize);
            universe += 1;
        }

        // Send any remaining dmx data.
        dmx_source
            .send(universe, &m.dmx.buffer[..])
            .expect("failed to send DMX data");
    }

    // Send our phase data over to the audio thead
    if let Some(ref audio_stream) = m.audio_stream {
        let phases = m.phases.clone();
        audio_stream
            .send(move |audio| {
                for (osc, phase) in audio.oscillators.iter_mut().zip(phases) {
                    osc.hz = phase as f64;
                }
            })
            .unwrap();
    }

    // Send our phase data over to the laser thead
    if let Some(ref laser_stream) = m.laser_stream {
        let phases = m.phases.clone();
        laser_stream
            .send(move |laser| {
                laser.positions.clear();
                laser.positions.extend(phases);
            })
            .unwrap();
    }
}

fn view(app: &App, m: &Model, frame: &Frame) {
    let draw = app.draw();
    draw.background().rgb(0.1, 0.1, 0.1);

    let win = app.window_rect();

    let radius = win.w() / m.shm.size() as f32;
    let height = win.h() / 2.0 - 20.0;

    m.phases.iter().enumerate().for_each(|(i, &phase)| {
        let x = map_range(i, 0, m.phases.len(), win.left(), win.right());

        draw.line()
            .color(WHITE)
            .start(Point2::new(x, 0.0))
            .end(Point2::new(x, phase * height));

        draw.ellipse()
            .color(WHITE)
            .x_y(x, phase * height)
            .w_h(radius, radius);
    });

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
    let points = laser.positions.iter().enumerate().map(|(i, y)| {
        let x = map_range(i, 0, laser.positions.len(), -1.0, 1.0);
        let pos = [x, *y];
        let rgb = [1.0, 1.0, 1.0];
        laser::Point::new(pos, rgb)
    });
    frame.add_lines(points);
}
