//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    cell::RefCell,
    collections::HashMap,
    io::{BufReader, Cursor},
    rc::Rc,
};

use fltk::{
    app,
    button::Button,
    enums::CallbackTrigger,
    frame::Frame,
    group::Flex,
    input::Input,
    prelude::*,
    valuator::{Slider, SliderType},
    window::Window,
};
use itertools::Itertools;
use rodio::{buffer::SamplesBuffer, Source};

use rand::prelude::*;

const ASCII_UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const ASCII_DIGITS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const WIDTH: i32 = 800;
const HEIGHT: i32 = 400;

// ToDo
//   0. (done, tts) record audios
//   1. (done) load every sound into hashmap with char/digit key
//   2. (done) hashmap values are vectors because 1 char can have multiple sounds
//   3. (done) debug: test out sounds with seperate buttons
//   4. (done) randomize callsigns (+length)
//   5. (done) generate sound from callsign text
//      py:
//         sounds = phonetics[rd.choice(string.ascii_lowercase)] // get phonetic sounds for char (or digit)
//         sound = rd.choice(sounds) // get one sound from multiple if any
//   6. (done) cache callsign audio (so it doesnt change each play)
//   7. (done) have user type in their answer and check it if match
//   8. (done) generate new callsigns
//   9. (half done) modify speed (random or user set)
//   10. mix randomised noise in
// Extras
//   - Real callsign from web with scraping/api
//   - Option to use custom sounds from folder (if sound x exists in sounds folder use that instead)
//   - Overlaping calls
//   - Location definers (/P, /M, etc)

// Callsign:
//   <1-2 char country code><1 number><1-4 char>

const SOUND_A: &[u8] = include_bytes!("sounds/a.mp3");
const SOUND_B: &[u8] = include_bytes!("sounds/b.mp3");
const SOUND_C: &[u8] = include_bytes!("sounds/c.mp3");
const SOUND_D: &[u8] = include_bytes!("sounds/d.mp3");
const SOUND_E: &[u8] = include_bytes!("sounds/e.mp3");
const SOUND_F: &[u8] = include_bytes!("sounds/f.mp3");
const SOUND_G: &[u8] = include_bytes!("sounds/g.mp3");
const SOUND_H: &[u8] = include_bytes!("sounds/h.mp3");
const SOUND_I: &[u8] = include_bytes!("sounds/i.mp3");
const SOUND_I2: &[u8] = include_bytes!("sounds/i2.mp3");
const SOUND_J: &[u8] = include_bytes!("sounds/j.mp3");
const SOUND_K: &[u8] = include_bytes!("sounds/k.mp3");
const SOUND_L: &[u8] = include_bytes!("sounds/l.mp3");
const SOUND_M: &[u8] = include_bytes!("sounds/m.mp3");
const SOUND_N: &[u8] = include_bytes!("sounds/n.mp3");
const SOUND_O: &[u8] = include_bytes!("sounds/o.mp3");
const SOUND_P: &[u8] = include_bytes!("sounds/p.mp3");
const SOUND_Q: &[u8] = include_bytes!("sounds/q.mp3");
const SOUND_R: &[u8] = include_bytes!("sounds/r.mp3");
const SOUND_S: &[u8] = include_bytes!("sounds/s.mp3");
const SOUND_T: &[u8] = include_bytes!("sounds/t.mp3");
const SOUND_U: &[u8] = include_bytes!("sounds/u.mp3");
const SOUND_V: &[u8] = include_bytes!("sounds/v.mp3");
const SOUND_W: &[u8] = include_bytes!("sounds/w.mp3");
const SOUND_X: &[u8] = include_bytes!("sounds/x.mp3");
const SOUND_Y: &[u8] = include_bytes!("sounds/y.mp3");
const SOUND_Z: &[u8] = include_bytes!("sounds/z.mp3");
const SOUND_0: &[u8] = include_bytes!("sounds/0.mp3");
const SOUND_1: &[u8] = include_bytes!("sounds/1.mp3");
const SOUND_2: &[u8] = include_bytes!("sounds/2.mp3");
const SOUND_3: &[u8] = include_bytes!("sounds/3.mp3");
const SOUND_4: &[u8] = include_bytes!("sounds/4.mp3");
const SOUND_5: &[u8] = include_bytes!("sounds/5.mp3");
const SOUND_6: &[u8] = include_bytes!("sounds/6.mp3");
const SOUND_7: &[u8] = include_bytes!("sounds/7.mp3");
const SOUND_8: &[u8] = include_bytes!("sounds/8.mp3");
const SOUND_9: &[u8] = include_bytes!("sounds/9.mp3");

struct Callsign {
    text: String,
    audio: Vec<SamplesBuffer<f32>>,
}

type Phonetics = HashMap<char, Rc<Vec<SamplesBuffer<f32>>>>;

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

fn generate_callsign(phonetics: &Phonetics) -> Callsign {
    let mut rd = rand::rng();

    let prefix_len = rd.random_range(1..=2);
    let prefix = generate_x_chars(prefix_len);

    let zone_num: u32 = rd.random_range(0..=9);

    let suffix_len = rd.random_range(1..=4);
    let suffix = generate_x_chars(suffix_len);

    let text = format!("{}{}{}", prefix, zone_num, suffix);

    let mut callsign = Callsign {
        text,
        audio: Vec::new(),
    };

    for c in callsign.text.clone().chars() {
        let sound = get_random_sound(phonetics.get(&c).unwrap()).clone();
        callsign.audio.push(sound);
    }

    println!("{}", callsign.text); // DEBUG

    callsign
}

fn load_phonetics() -> Phonetics {
    let mut phonetics = HashMap::new();

    let all_sounds: [Vec<&[u8]>; 36] = [
        vec![SOUND_A],
        vec![SOUND_B],
        vec![SOUND_C],
        vec![SOUND_D],
        vec![SOUND_E],
        vec![SOUND_F],
        vec![SOUND_G],
        vec![SOUND_H],
        vec![SOUND_I, SOUND_I2],
        vec![SOUND_J],
        vec![SOUND_K],
        vec![SOUND_L],
        vec![SOUND_M],
        vec![SOUND_N],
        vec![SOUND_O],
        vec![SOUND_P],
        vec![SOUND_Q],
        vec![SOUND_R],
        vec![SOUND_S],
        vec![SOUND_T],
        vec![SOUND_U],
        vec![SOUND_V],
        vec![SOUND_W],
        vec![SOUND_X],
        vec![SOUND_Y],
        vec![SOUND_Z],
        vec![SOUND_0],
        vec![SOUND_1],
        vec![SOUND_2],
        vec![SOUND_3],
        vec![SOUND_4],
        vec![SOUND_5],
        vec![SOUND_6],
        vec![SOUND_7],
        vec![SOUND_8],
        vec![SOUND_9],
    ];

    for (sounds, character) in all_sounds.iter().zip(ASCII_DIGITS.chars()) {
        let mut buffered_sounds: Vec<SamplesBuffer<f32>> = Vec::new();
        for sound in sounds {
            buffered_sounds.push(get_playable_audio(sound));
        }
        phonetics.insert(character, Rc::new(buffered_sounds));
    }

    phonetics
}

fn get_random_sound(sounds: &Vec<SamplesBuffer<f32>>) -> &SamplesBuffer<f32> {
    let mut rd = rand::rng();
    sounds.choose(&mut rd).unwrap()
}

fn main() {
    let phonetics = load_phonetics();

    let callsign = Rc::new(RefCell::new(generate_callsign(&phonetics)));

    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = Rc::new(rodio::Sink::try_new(&handle).unwrap());
    let handle_rc = Rc::new(handle);

    let app = app::App::default();
    let mut wind = Window::new(0, 0, WIDTH, HEIGHT, "Callsign simulator").center_screen();

    let test_flex = Flex::new(0, 0, WIDTH, 30, "").row();
    for (character, sounds) in phonetics.iter().sorted_by_key(|x| x.0) {
        let mut but = Button::new(0, 0, 20, 0, character.to_string().as_str());
        let sounds_clone = Rc::clone(&sounds);
        let handle_clone = Rc::clone(&handle_rc);
        but.set_callback(move |_| {
            let sound = get_random_sound(&sounds_clone).clone();
            handle_clone.play_raw(sound).unwrap();
        });
    }
    test_flex.end();

    let flex = Flex::new(0, HEIGHT / 2, WIDTH, HEIGHT / 2, "").column();
    let output_frame = Rc::new(RefCell::new(Frame::new(0, 0, WIDTH, HEIGHT / 4, "")));

    let but_flex = Flex::new(0, 0, 0, 40, "").row();
    let mut check_but = Button::new(0, 0, 80, 40, "Check input");
    let mut play_but = Button::new(0, 0, 80, 40, "Play callsign");
    let mut new_but = Button::new(0, 0, 80, 40, "New callsign");
    but_flex.end();

    let callsign_input = Rc::new(RefCell::new(Input::new(0, 0, 80, 40, "")));
    let mut speed_slider = Slider::new(0, 0, 0, 20, "speed");
    speed_slider.set_type(SliderType::HorizontalNice);
    flex.end();

    wind.end();
    wind.make_resizable(true);
    wind.show();

    let output_frame_clone = Rc::clone(&output_frame);
    let sink_clone = Rc::clone(&sink);
    let callsign_clone = Rc::clone(&callsign);
    new_but.set_callback(move |_| {
        if !sink_clone.empty() {
            output_frame_clone
                .borrow_mut()
                .set_label("Cannot generate new callsign while playing current");
            return;
        }

        output_frame_clone
            .borrow_mut()
            .set_label("Generated new callsign");

        *callsign_clone.borrow_mut() = generate_callsign(&phonetics);
    });

    let callsign_clone = Rc::clone(&callsign);
    let input_clone = Rc::clone(&callsign_input);
    let output_frame_clone = Rc::clone(&output_frame);
    check_but.set_callback(move |_| {
        if input_clone.borrow().value().to_ascii_uppercase() == callsign_clone.borrow().text {
            output_frame_clone
                .borrow_mut()
                .set_label("Correct\nGenerated new callsign");
            new_but.do_callback();
        } else {
            output_frame_clone.borrow_mut().set_label("Wrong");
        }
    });

    let output_frame_clone = Rc::clone(&output_frame);
    let callsign_clone = Rc::clone(&callsign);
    let sink_clone = Rc::clone(&sink);
    play_but.set_callback(move |_| {
        if !sink_clone.empty() {
            return;
        }

        output_frame_clone
            .borrow_mut()
            .set_label("Playing callsign...");

        for e in callsign_clone.borrow().audio.iter() {
            sink_clone.append(e.clone());
        }
    });

    callsign_input
        .borrow_mut()
        .set_trigger(CallbackTrigger::EnterKey);
    callsign_input.borrow_mut().set_callback(move |_| {
        check_but.do_callback();
    });

    let sink_clone = Rc::clone(&sink);
    speed_slider.set_callback(move |s| {
        sink_clone.set_speed(s.value() as f32 + 1.0);
    });

    app.run().unwrap();
}
