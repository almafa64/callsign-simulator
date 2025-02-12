//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::{BufReader, Cursor};

use fltk::{
    app, button::Button, enums::CallbackTrigger, frame::Frame, group::Flex, input::Input,
    prelude::*, window::Window,
};
use rodio::{buffer::SamplesBuffer, Source};

use rand::prelude::*;

const ASCII_UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

const MUSIC_FILE: &[u8] = include_bytes!("audio.mp3");

// ToDo
//   1. load every sound into hashmap with char/digit key
//   2. hashmap values are arrays because 1 char can have multiple sounds
//   3. debug: test out sounds with seperate buttons
//   4. (done) randomize callsigns (+length)
//   5. generate sound from callsign text
//      py:
//         sounds = phonetics[rd.choice(string.ascii_lowercase)] // get phonetic sounds for char (or digit)
//         sound = rd.choice(sounds) // get one sound from multiple if any
//   6. have user type in their answer and check it if match
//   7. mix randomised noise in
// Extras
//   - Real callsign from web with scraping/api
//   - Option to use custom sounds from folder (if sound x exists in sounds folder use that instead)
//   - Overlaping calls

// Callsign:
//   <1-2 char country code><1 number><1-4 char>

/*
const SOUND_A: &[u8]    = include_bytes!("sounds/a.mp3");
const SOUND_B: &[u8]    = include_bytes!("sounds/b.mp3");
const SOUND_C: &[u8]    = include_bytes!("sounds/c.mp3");
const SOUND_D: &[u8]    = include_bytes!("sounds/d.mp3");
const SOUND_E: &[u8]    = include_bytes!("sounds/e.mp3");
const SOUND_F: &[u8]    = include_bytes!("sounds/f.mp3");
const SOUND_G: &[u8]    = include_bytes!("sounds/g.mp3");
const SOUND_H: &[u8]    = include_bytes!("sounds/h.mp3");
const SOUND_I: &[u8]    = include_bytes!("sounds/i.mp3");
const SOUND_I2: &[u8]   = include_bytes!("sounds/i2.mp3");
const SOUND_J: &[u8]    = include_bytes!("sounds/j.mp3");
const SOUND_K: &[u8]    = include_bytes!("sounds/k.mp3");
const SOUND_L: &[u8]    = include_bytes!("sounds/l.mp3");
const SOUND_M: &[u8]    = include_bytes!("sounds/m.mp3");
const SOUND_N: &[u8]    = include_bytes!("sounds/n.mp3");
const SOUND_O: &[u8]    = include_bytes!("sounds/o.mp3");
const SOUND_P: &[u8]    = include_bytes!("sounds/p.mp3");
const SOUND_Q: &[u8]    = include_bytes!("sounds/q.mp3");
const SOUND_R: &[u8]    = include_bytes!("sounds/r.mp3");
const SOUND_S: &[u8]    = include_bytes!("sounds/s.mp3");
const SOUND_T: &[u8]    = include_bytes!("sounds/t.mp3");
const SOUND_U: &[u8]    = include_bytes!("sounds/u.mp3");
const SOUND_V: &[u8]    = include_bytes!("sounds/v.mp3");
const SOUND_W: &[u8]    = include_bytes!("sounds/w.mp3");
const SOUND_X: &[u8]    = include_bytes!("sounds/x.mp3");
const SOUND_Y: &[u8]    = include_bytes!("sounds/y.mp3");
const SOUND_Z: &[u8]    = include_bytes!("sounds/z.mp3");

const SOUND_0: &[u8]    = include_bytes!("sounds/0.mp3");
const SOUND_1: &[u8]    = include_bytes!("sounds/1.mp3");
const SOUND_2: &[u8]    = include_bytes!("sounds/2.mp3");
const SOUND_3: &[u8]    = include_bytes!("sounds/3.mp3");
const SOUND_4: &[u8]    = include_bytes!("sounds/4.mp3");
const SOUND_5: &[u8]    = include_bytes!("sounds/5.mp3");
const SOUND_6: &[u8]    = include_bytes!("sounds/6.mp3");
const SOUND_7: &[u8]    = include_bytes!("sounds/7.mp3");
const SOUND_8: &[u8]    = include_bytes!("sounds/8.mp3");
const SOUND_9: &[u8]    = include_bytes!("sounds/9.mp3");
*/

fn get_playable_audio(audio_byte_array: &'static [u8]) -> SamplesBuffer<f32> {
    let cursor = Cursor::new(audio_byte_array);
    let decoder = rodio::Decoder::new_mp3(BufReader::new(cursor)).unwrap();
    let (channels, sample_rate) = (decoder.channels(), decoder.sample_rate());
    let samples: Vec<f32> = decoder.convert_samples().collect();
    rodio::buffer::SamplesBuffer::new(channels, sample_rate, samples)
}

fn generate_x_chars(count: usize) -> String {
    let mut rd = rand::rng();
    ASCII_UPPERCASE
        .chars()
        .choose_multiple(&mut rd, count)
        .iter()
        .collect()
}

fn generate_callsign() -> String {
    let mut rd = rand::rng();

    let prefix_len = rd.random_range(1..=2);
    let prefix = generate_x_chars(prefix_len);

    let zone_num: u32 = rd.random_range(0..=9);

    let suffix_len = rd.random_range(1..=4);
    let suffix = generate_x_chars(suffix_len);

    format!("{}{}{}", prefix, zone_num, suffix)
}

fn main() {
    let callsign = generate_callsign();
    println!("{}", callsign);

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
        "Callsign simulator",
    );
    let mut flex = Flex::new(0, height / 2, width, height / 2, "").column();
    let mut frame = Frame::new(0, 0, width, height / 2, "");
    let mut but = Button::new(160, 210, 80, 40, "Check input");
    let mut input = Input::new(0, 0, 80, 40, "");
    flex.end();
    wind.end();
    wind.make_resizable(true);
    wind.show();
    but.set_callback(move |_| {
        //sink.append(buf.clone());
        //handle.play_raw(buf.clone()).unwrap();
        if input.value().to_ascii_uppercase() == callsign {
            frame.set_label("Good");
        } else {
            frame.set_label("Bad");
        }
    });
    /*input.set_trigger(CallbackTrigger::Changed);
    input.set_callback(move |x| {
        frame.set_label(&x.value());
    });*/
    app.run().unwrap();
}
