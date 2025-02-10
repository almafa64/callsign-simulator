//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::{BufReader, Cursor};

use fltk::{
    app, button::Button, enums::CallbackTrigger, frame::Frame, group::Flex, input::Input,
    prelude::*, window::Window,
};
use rodio::{buffer::SamplesBuffer, Source};

const MUSIC_FILE: &[u8] = include_bytes!("audio.mp3");

fn get_playable_audio(audio_byte_array: &'static [u8]) -> SamplesBuffer<f32> {
    let cursor = Cursor::new(audio_byte_array);
    let decoder = rodio::Decoder::new_mp3(BufReader::new(cursor)).unwrap();
    let (channels, sample_rate) = (decoder.channels(), decoder.sample_rate());
    let samples: Vec<f32> = decoder.convert_samples().collect();
    rodio::buffer::SamplesBuffer::new(channels, sample_rate, samples)
}

fn main() {
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();

    let buf = get_playable_audio(MUSIC_FILE);

    let app = app::App::default();
    let (mon_width, mon_height) = app::screen_size();
    let (width, height) = (400, 400);
    let mut wind = Window::new(
        (mon_width / 2.0 - (width as f64) / 2.0) as i32,
        (mon_height / 2.0 - (height as f64) / 2.0) as i32,
        width,
        height,
        "Hello from rust",
    );
    let mut flex = Flex::new(0, height / 2, width, height / 2, "").column();
    let mut frame = Frame::new(0, 0, width, height / 2, "");
    let mut but = Button::new(160, 210, 80, 40, "Click me!");
    let mut input = Input::new(0, 0, 80, 40, "");
    flex.end();
    wind.end();
    wind.make_resizable(true);
    wind.show();
    but.set_callback(move |_| {
        handle.play_raw(buf.clone()).unwrap();
    });
    input.set_trigger(CallbackTrigger::Changed);
    input.set_callback(move |x| {
        frame.set_label(&x.value());
    });
    app.run().unwrap();
}
