use nannou::ui::conrod_core::widget_ids;
use nannou::ui::prelude::*;
use nannou::ui::Color;
use super::SignalParams;
use crate::LED_PIXELS_PER_UNIVERSE;
use crate::shm::Shm;

pub const PAD: Scalar = 20.0;
pub const WIDGET_W: Scalar = 200.0;
pub const COLUMN_W: Scalar = WIDGET_W + PAD * 2.0;
pub const DEFAULT_WIDGET_H: Scalar = 30.0;
pub const DEFAULT_SLIDER_H: Scalar = 20.0;
pub const HALF_WIDGET_W: Scalar = WIDGET_W * 0.5 - PAD * 0.25;
pub const THIRD_WIDGET_W: Scalar = WIDGET_W * 0.33 - PAD * 0.25;
pub const WIDGET_COLOUR: Color = Color::Rgba(0.98, 0.1, 0.28, 1.0);

widget_ids! {
    pub struct Ids {
        background,
        title_text,
        min_max,
        speed,
        offset,
        offset_hz,
        pow,
        skew,
        count,
        mirror,
        invert,
        signal_type,
        dmx_on,
        laser_on,
        audio_on,
    }
}

/// Update the user interface.
pub fn update(
    ref mut ui: UiCell,
    ids: &mut Ids,
    params: &mut SignalParams,
    shm: &mut Shm,
) {
    widget::Canvas::new()
        .pad(PAD)
        .border(0.0)
        .w(COLUMN_W)
        .top_left()
        .rgba(0.2, 0.2, 0.2, 0.5)
        .set(ids.background, ui);

    text("NANNOU SIGNALS")
        .mid_top_of(ids.background)
        .set(ids.title_text, ui);

    for value in toggle(params.dmx_on)
        .mid_left_of(ids.background)
        .down(20.0)
        .w(THIRD_WIDGET_W)
        .label("DMX")
        .set(ids.dmx_on, ui)
    {
        params.dmx_on = value;
    }

    for value in toggle(params.laser_on)
        .right(PAD * 0.4)
        .w(THIRD_WIDGET_W)
        .label("LASER")
        .set(ids.laser_on, ui)
    {
        params.laser_on = value;
    }

    for value in toggle(params.audio_on)
        .right(PAD * 0.4)
        .w(THIRD_WIDGET_W)
        .label("AUDIO")
        .set(ids.audio_on, ui)
    {
        params.audio_on = value;
    }

    let min = -1.0;
    let max = 1.0;
    for (edge, value) in widget::RangeSlider::new(params.min, params.max, min, max)
        .color(WIDGET_COLOUR)
        .label("Amplitude")
        .mid_left_of(ids.background)
        .down(10.0)
        .label_font_size(14)
        .label_rgb(1.0, 1.0, 1.0)
        .w_h(200.0, 20.0)
        .set(ids.min_max, ui)
    {
        match edge {
            widget::range_slider::Edge::Start => params.min = value,
            widget::range_slider::Edge::End => params.max = value,
        }
    }

    for value in slider(shm.hz, -1.0, 1.0)
        .down(10.0)
        .label("Speed")
        .set(ids.speed, ui)
    {
        shm.hz = value;
    }

    for value in slider(shm.a_velocity, 0.0, 1.0)
        .down(10.0)
        .label("Offset")
        .set(ids.offset, ui)
    {
        shm.a_velocity = value;
    }

    for value in slider(shm.offset_hz, 0.0, 1.0)
        .down(10.0)
        .label("Offset Hz")
        .set(ids.offset_hz, ui)
    {
        shm.offset_hz = value;
    }

    for value in slider(params.pow, 0.5, 10.0)
        .down(10.0)
        .label("Pow")
        .set(ids.pow, ui)
    {
        params.pow = value;
    }

    for value in slider(shm.skew, 0.25, 10.0)
        .down(10.0)
        .label("Skew")
        .set(ids.skew, ui)
    {
        shm.skew = value;
    }

    for value in slider(shm.size() as f32, 1.0, LED_PIXELS_PER_UNIVERSE as f32)
        .down(10.0)
        .label("Count")
        .set(ids.count, ui)
    {
        shm.set_size(value as _);

        //m.stream.send(move |audio| { audio.oscillators.resize(value as usize, Oscillator{phase: 0.0, hz: 100.0}); }).unwrap();
    }

    for value in toggle(shm.mirror)
        .down(10.0)
        .w(HALF_WIDGET_W)
        .label("Mirror")
        .set(ids.mirror, ui)
    {
        shm.mirror = value;
    }

    for value in toggle(params.invert)
        .right(10.0)
        .w(HALF_WIDGET_W)
        .label("Invert")
        .set(ids.invert, ui)
    {
        params.invert = value;
    }

    for selected_idx in widget::DropDownList::new(&params.signal_names, params.selected_idx)
        .w_h(WIDGET_W, DEFAULT_WIDGET_H)
        .down_from(ids.mirror, 10.0)
        .max_visible_items(10)
        .color(WIDGET_COLOUR)
        .label("Signal Type")
        .label_font_size(14)
        .label_rgb(1.0, 1.0, 1.0)
        .scrollbar_on_top()
        .set(ids.signal_type, ui)
    {
        params.selected_idx = Some(selected_idx);
    }
}

fn slider(val: f32, min: f32, max: f32) -> widget::Slider<'static, f32> {
    widget::Slider::new(val, min, max)
        .w_h(WIDGET_W, DEFAULT_SLIDER_H)
        .label_font_size(14)
        .color(WIDGET_COLOUR)
        .label_rgb(1.0, 1.0, 1.0)
        .border(0.0)
}

// Shorthand for the toggle style we'll use.
fn toggle(b: bool) -> widget::Toggle<'static> {
    widget::Toggle::new(b)
        .w_h(COLUMN_W, DEFAULT_WIDGET_H)
        .label_font_size(14)
        .color(WIDGET_COLOUR)
        .label_rgb(1.0, 1.0, 1.0)
        .border(0.0)
}

fn text(s: &str) -> widget::Text {
    widget::Text::new(s).color(color::WHITE)
}
