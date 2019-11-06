mod shm;
mod signals;

use std::thread;
use nannou_audio::{self, Buffer};
use nannou::prelude::*;
use nannou::ui::conrod_core::widget_ids;
use nannou::ui::prelude::*;
use nannou::Ui;
use shm::Shm;
use signals::Signal;
use ether_dream::{dac, protocol::DacPoint};
use std::sync::mpsc;


fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    dmx: Dmx,
    //stream: audio::Stream<Audio>,
    shm: Shm,
    ui: Ui,
    ids: gui::Ids,
    signal_names: Vec<String>,
    selected_idx: Option<usize>,
    pow: f32,
    min: f32,
    max: f32,
    invert: bool, // Invert all of the phases
    //laser_tx: mpsc::Sender<Vec<DacPoint>>,
}

widget_ids! {
    struct Ids {
        min_max: widget::Id,
        speed: widget::Id,
        offset: widget::Id,
        offset_hz: widget::Id,
        pow: widget::Id,
        skew: widget::Id,
        count: widget::Id,
        mirror: widget::Id,
        invert: widget::Id,
        signal_type: widget::Id,
    }
}

#[derive(Clone)]
struct Oscillator {
    phase: f64,
    hz: f64,
}

struct Audio {
    oscillators: Vec<Oscillator>,
}

struct Dmx {
    source: Option<sacn::DmxSource>,
    buffer: Vec<u8>,
}

fn model(app: &App) -> Model {
    
//----------------------------------------------------- Laser INIT
    println!("Listening for an Ether Dream DAC...");

    // let (dac_broadcast, source_addr) = ether_dream::recv_dac_broadcasts()
    //     .expect("failed to bind to UDP socket")
    //     .filter_map(Result::ok)
    //     .next()
    //     .unwrap();
    // let mac_address = dac::MacAddress(dac_broadcast.mac_address);

    // println!("Discovered DAC \"{}\" at \"{}\"! Connecting...", mac_address, source_addr);

    // // Establish the TCP connection.
    // let mut laser_stream = dac::stream::connect(&dac_broadcast, source_addr.ip().clone()).unwrap();

    // // If we want to create an animation (in our case a moving sine wave) we need a frame rate.
    // let frames_per_second = 60.0;
    // // Lets use the DAC at an eighth the maximum scan rate.
    // let points_per_second = laser_stream.dac().max_point_rate / 32;
    // // Determine the number of points per frame given our target frame and point rates.
    // let points_per_frame = (points_per_second as f32 / frames_per_second) as u16;

    // println!("Preparing for playback:\n\tframe_hz: {}\n\tpoint_hz: {}\n\tpoints_per_frame: {}\n",
    //             frames_per_second, points_per_second, points_per_frame);

    // // Prepare the DAC's playback engine and await the repsonse.
    // laser_stream
    //     .queue_commands()
    //     .prepare_stream()
    //     .submit()
    //     .err()
    //     .map(|err| {
    //         eprintln!("err occurred when submitting PREPARE_STREAM \
    //                     command and listening for response: {}", err);
    //     });

    // println!("Beginning playback!");

    // // The sine wave used to generate points.
    // let mut sine_wave = SineWave { point: 0, points_per_frame, frames_per_second };

    // // Queue the initial frame and tell the DAC to begin producing output.
    // let n_points = points_to_generate(laser_stream.dac());;
    // laser_stream
    //     .queue_commands()
    //     .data(sine_wave.by_ref().take(n_points))
    //     .begin(0, points_per_second)
    //     .submit()
    //     .err()
    //     .map(|err| {
    //         eprintln!("err occurred when submitting initial DATA and BEGIN \
    //                     commands and listening for response: {}", err);
    //     });

    // // Loop and continue to send points forever.
    // let (laser_tx, laser_rx) = mpsc::channel();

    // let thread = thread::Builder::new()
    //     .name("ccxt::all_exchange_ticks".into())
    //     .stack_size(10_000) // 10kb, only need a small stack.
    //     .spawn(move || {
    //         let mut streamer = laser_frame::Streamer::new();
    //         loop {
    //             if let Some(frame) = laser_rx.try_iter().last() {
    //                 streamer.submit_frame(frame);
    //             }
    //             // Determine how many points the DAC can currently receive.
    //             let n_points = points_to_generate(laser_stream.dac());
    //             if let Err(err) = laser_stream
    //                 .queue_commands()
    //                 .data(streamer.next_points().take(n_points).map(|p: laser_frame::Point<DacPoint>| {
    //                     match p {
    //                         laser_frame::Point::Blank(mut p) => {
    //                             p.r = 0;
    //                             p.g = 0;
    //                             p.b = 0;
    //                             p
    //                         }
    //                         laser_frame::Point::Regular(p) => p,
    //                     }
    //                 }))
    //                 .submit()
    //             {
    //                 eprintln!("err occurred when submitting DATA command and listening \
    //                             for response: {}", err);
    //                 //break;
    //             }
    //         }
    //     })
    //     .unwrap();

    //app.set_loop_mode(LoopMode::wait(3));
    //let _window = app.new_window().with_dimensions(1280, 800).build().unwrap();
    
    
    //let mut shm = Shm::new(points_per_frame as usize, 0.1, 0.49, 0.0);
    let mut shm = Shm::new(34, 0.1, 0.49, 0.0);
    let pow = 1.0;
    let min = -1.0;
    let max = 1.0;
    let invert = false;
    let signal_names = signals::Signal::all_names();
    let selected_idx = None;

    shm.set_signal_type(Signal::SINE_IN_OUT);

    // Create the UI
    let mut ui = app.new_ui().build().unwrap();

    // Generate some ids for our widgets
    let ids = Ids {
        min_max: ui.generate_widget_id(),
        speed: ui.generate_widget_id(),
        offset: ui.generate_widget_id(),
        offset_hz: ui.generate_widget_id(),
        pow: ui.generate_widget_id(),
        skew: ui.generate_widget_id(),
        count: ui.generate_widget_id(),
        mirror: ui.generate_widget_id(),
        invert: ui.generate_widget_id(),
        signal_type: ui.generate_widget_id(),
    };


    // Initialise the state that we want to live on the audio thread.
    let oscillators = (0..shm.size()).map(|_| Oscillator{phase: 0.0, hz: 100.0}).collect();
    let model = Audio {
        oscillators,
    };
    //let stream = app.audio.new_output_stream(model, audio).build().unwrap();

    let dmx = DmxSource::new("Nannou DMX").unwrap();
    //let dmx = DmxSource::with_ip("ss", "192.168.0.100").unwrap();

    Model {
        dmx,
        //stream,
        shm,
        ui,
        ids,
        signal_names,
        selected_idx,
        pow,
        min,
        max,
        invert,
        //laser_tx,
    }
}

// Determine the number of points needed to fill the DAC.
fn points_to_generate(dac: &ether_dream::dac::Dac) -> usize {
    dac.buffer_capacity as usize - 1 - dac.status.buffer_fullness as usize
}

// An iterator that endlessly generates a sine wave of DAC points.
//
// The sine wave oscillates at a rate of once per second.
struct SineWave {
    point: u32,
    points_per_frame: u16,
    frames_per_second: f32,
}

impl Iterator for SineWave {
    type Item = ether_dream::protocol::DacPoint;
    fn next(&mut self) -> Option<Self::Item> {
        let coloured_points_per_frame = self.points_per_frame - 1;
        let i = (self.point % self.points_per_frame as u32) as u16;
        let hz = 1.0;
        let fract = i as f32 / coloured_points_per_frame as f32;
        let phase = (self.point as f32 / coloured_points_per_frame as f32) / self.frames_per_second;
        let amp = (hz * (fract + phase) * 2.0 * std::f32::consts::PI).sin();
        let (r, g, b) = match i {
            i if i == coloured_points_per_frame || i < 13 => (0, 0, 0),
            _ => (std::u16::MAX, std::u16::MAX, std::u16::MAX),
        };
        let x_min = std::i16::MIN;
        let x_max = std::i16::MAX;
        let x = (x_min as f32 + fract * (x_max as f32 - x_min as f32)) as i16;
        let y = (amp * x_max as f32) as i16;
        let control = 0;
        let (u1, u2) = (0, 0);
        let p = ether_dream::protocol::DacPoint { control, x, y, i, r, g, b, u1, u2 };
        self.point += 1;
        Some(p)
    }
}

fn update(app: &App, m: &mut Model, update: Update) {
    // Calling 'set widgets' allows us to instantiate some widgets.
    let ui = &mut m.ui.set_widgets();

    let min = -1.0;
    let max = 1.0;
    for (edge, value) in widget::RangeSlider::new(m.min, m.max, min, max)
        .rgb(0.3, 0.3, 0.3)
        .label("Amplitude")
        .top_left_with_margin(20.0)
        .label_font_size(15)
        .label_rgb(1.0, 1.0, 1.0)
        .w_h(200.0, 30.0)
        .set(m.ids.min_max, ui)
    {
        match edge {
            widget::range_slider::Edge::Start => m.min = value,
            widget::range_slider::Edge::End => m.max = value,
        }
    }

    fn slider(val: f32, min: f32, max: f32) -> widget::Slider<'static, f32> {
        widget::Slider::new(val, min, max)
            .w_h(200.0, 30.0)
            .label_font_size(15)
            .rgb(0.3, 0.3, 0.3)
            .label_rgb(1.0, 1.0, 1.0)
    }

    for value in slider(m.shm.hz, -1.0, 1.0)
        .down(10.0)
        .label("Speed")
        .set(m.ids.speed, ui)
    {
        m.shm.hz = value;
    }

    for value in slider(m.shm.a_velocity, 0.0, 1.0)
        .down(10.0)
        .label("Offset")
        .set(m.ids.offset, ui)
    {
        m.shm.a_velocity = value;
    }

    m.shm.a_velocity = map_range((app.time * 0.2).sin(), -1.0, 1.0, 0.94, 1.0);

    for value in slider(m.shm.offset_hz, 0.0, 1.0)
        .down(10.0)
        .label("Offset Hz")
        .set(m.ids.offset_hz, ui)
    {
        m.shm.offset_hz = value;
    }

    for value in slider(m.pow, 0.5, 10.0)
        .down(10.0)
        .label("Pow")
        .set(m.ids.pow, ui)
    {
        m.pow = value;
    }

    for value in slider(m.shm.skew, 0.25, 10.0)
        .down(10.0)
        .label("Skew")
        .set(m.ids.skew, ui)
    {
        m.shm.skew = value;
    }

    for value in slider(m.shm.size() as f32, 1.0, 512.0)
        .down(10.0)
        .label("Count")
        .set(m.ids.count, ui)
    {
        m.shm.set_size(value as _);

        //m.stream.send(move |audio| { audio.oscillators.resize(value as usize, Oscillator{phase: 0.0, hz: 100.0}); }).unwrap();
    }

    for value in widget::Toggle::new(m.shm.mirror)
        .down(10.0)
        .w_h(95.0, 30.0)
        .label("Mirror")
        .label_font_size(15)
        .rgb(0.3, 0.3, 0.3)
        .label_rgb(1.0, 1.0, 1.0)
        .set(m.ids.mirror, ui)
    {
        m.shm.mirror = value;
    }

    for value in widget::Toggle::new(m.invert)
        .right(10.0)
        .w_h(95.0, 30.0)
        .label("Invert")
        .label_font_size(15)
        .rgb(0.3, 0.3, 0.3)
        .label_rgb(1.0, 1.0, 1.0)
        .set(m.ids.invert, ui)
    {
        m.invert = value;
    }

    // A demonstration using a DropDownList to select its own color.
    for selected_idx in widget::DropDownList::new(&m.signal_names, m.selected_idx)
        .w_h(200.0, 30.0)
        .down_from(m.ids.mirror, 10.0)
        .max_visible_items(10)
        .rgb(0.3, 0.3, 0.3)
        .label("Signal Type")
        .label_font_size(15)
        .label_rgb(1.0, 1.0, 1.0)
        .scrollbar_on_top()
        .set(m.ids.signal_type, ui)
    {
        m.selected_idx = Some(selected_idx);
        m.shm.set_signal_type(signals::ALL[selected_idx]);
    }

    m.shm.update();

    // for i in 0..m.shm.size() {
    //     let mut phase = map_range(m.shm.phases()[i], -1.0, 1.0, 0.0, 1.0);
    //     phase = phase.powf(m.pow);
    //     phase = match m.invert {
    //         true => map_range(phase, 0.0, 1.0, m.max, m.min),
    //         false => map_range(phase, 0.0, 1.0, m.min, m.max),
    //     };
    //     phase = map_range(phase, -1.0, 1.0, 50.0, 5000.0);
    //     //phase = map_range(phase.powf(m.pow), 50.0 * m.pow, 10000.0 * m.pow, 50.0, 10000.0);

    //     m.stream.send(move |audio| { audio.oscillators[i].hz = phase as f64; }).unwrap();
    // }        
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, m: &Model, frame: &Frame) {
    
    // Begin drawing
    let draw = app.draw();

    draw.background().rgb(0.02, 0.902, 0.402);

    // Get our windows rect
    let win = app.window_rect();

    let mut dmx_data = Vec::new();
    //let mut laser_data = Vec::new();

    let radius = win.w() / m.shm.size() as f32;
    let height = 300.0;
    for i in 0..m.shm.size() {
        let x = (win.left() + (radius * 0.5)) + i as f32 * radius;
        let mut phase = map_range(m.shm.phases()[i], -1.0, 1.0, 0.0, 1.0);
        phase = phase.powf(m.pow);

        let y = match m.invert {
            true => map_range(phase, 0.0, 1.0, m.max, m.min) * height,
            false => map_range(phase, 0.0, 1.0, m.min, m.max) * height,
        };
        
        dmx_data.push(map_range(y, 0.0, height, 0.0, 255.0) as u8);

        let (h, s, v) = (1.0 - (i as f32 / m.shm.size() as f32), 0.75, 0.5);

        draw.line()
            .start(Point2::new(x, 0.0))
            .end(Point2::new(x, y))
            .hsv(h, s, v);

        draw.ellipse().x_y(x, y).w_h(radius, radius).hsv(h, s, v);

        //println!("x {}", x);
        // let (u1, u2) = (0, 0);
        // let control = 0;
        // let x: i16 = map_range(clamp(x,win.left(),win.right()),win.left(), win.right(),std::i16::MIN, std::i16::MAX);

        // let (r,g,b) = nannou::ui::color::hsl_to_rgb(h,s,v);

        // //println!("x amp {}", x);
        // let y: i16  = map_range(clamp(y,0.0, height),0.0, height,std::i16::MIN, std::i16::MAX);
        // let r: u16 = 60535;// map_range(r,0.0,1.0,1,60535);
        // let g: u16 = map_range(g,0.0,1.0,1,60535);
        // let b: u16 = map_range(b,0.0,1.0,1,60535);
        // let ppf: u16 = i as u16;
        // let p = ether_dream::protocol::DacPoint { control, x, y, i: ppf, r, g: g / 100, b: b / 100, u1, u2 };
        // laser_data.push(p);
    }

    //m.laser_tx.send(laser_data);
    
    // Send our array of dmx data to the LEDs
    m.dmx.send(1, &dmx_data).unwrap();

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Draw the UI
    m.ui.draw_to_frame(app, &frame).unwrap();
}

// A function that renders the given `Audio` to the given `Buffer`, returning the result of both.
// In this case we play a simple sine wave at the audio's current frequency in `hz`..
fn audio(mut audio: Audio, mut buffer: Buffer) -> (Audio, Buffer) {
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
    (audio, buffer)
}