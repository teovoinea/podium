use imgui::*;
use glium::{
    backend::Facade,
    texture::{ClientFormat, RawImage2d},
    Texture2d,
};
use image::{jpeg::JPEGDecoder, ImageDecoder};

use std::borrow::Cow;
use std::error::Error;
use std::io::Cursor;
use std::sync::mpsc::{Receiver, Sender};
use std::process::Command;
use std::path::Path;

use super::support::Textures;
use super::support::RunState;
use crate::query_executor::{ QueryResponse, Response };

const CLEAR_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const WINDOW_BG: [f32; 4] = [0.27, 0.27, 0.28, 1.0];
const TEXT_COLOR: [f32; 4] = [0.8, 0.8, 0.81, 1.0];

struct State {
    query: ImString,
    icon_texture: Option<Icon>,
    response: Option<QueryResponse>,
}

const SEARCH_SIZE: (f32, f32) = (680.0, 60.0);
const RESULTS_SIZE: (f32, f32) = (680.0, 500.0);

pub fn run_window(query_sender: Sender<String>, results_receiver: Receiver<QueryResponse>) {
    let mut state = State {
        query: ImString::with_capacity(128),
        icon_texture: None,
        response: None
    };
    super::support::run("hello_world.rs".to_owned(), CLEAR_COLOR, |ui, gl_ctx, textures| {
        state.icon_texture = Some(Icon::new(gl_ctx, textures).unwrap());
        hello_world(&mut state, ui, &query_sender, &results_receiver)
    });
}

fn hello_world<'a>(state: &mut State, ui: &Ui<'a>, query_sender: &Sender<String>, response_receiver: &Receiver<QueryResponse>) -> RunState {
    let size = if state.response.is_some() {
        RESULTS_SIZE
    }
    else {
        SEARCH_SIZE
    };

    ui.with_color_var(ImGuiCol::WindowBg, WINDOW_BG, || {
        ui.window(im_str!("Search box"))
        .position((0.0, 0.0), ImGuiCond::Always)
        .size(size, ImGuiCond::Always)
        .title_bar(false)
        .inputs(true)
        .always_use_window_padding(false)
        .scroll_bar(false)
        .scrollable(false)
        .resizable(false)
        .movable(false)
        .collapsible(false)
        .build(|| {
            if let Some(icon) = &state.icon_texture {
                icon.show(&ui);
            }

            ui.same_line(0.0);
            ui.text_colored(TEXT_COLOR, im_str!("Search..."));
            
            ui.same_line(0.0);
            ui.with_color_var(ImGuiCol::FrameBg, WINDOW_BG, || {
                if ui.input_text(im_str!(""), &mut state.query)
                    .enter_returns_true(true)
                    .build()
                {
                    ui.set_keyboard_focus_here(-1);
                    if !state.query.to_str().trim().is_empty() {
                        dbg!(&state.query.to_str());
                        query_sender.send(String::from(state.query.to_str()));
                        let resp = response_receiver.recv().unwrap();
                        dbg!(&resp);
                        state.response = Some(resp);
                    }
                    else {
                        state.response = None;
                    }
                }
            });

            if let Some(responses) = &state.response {
                ui.separator();
                ui.child_frame(im_str!("Results box"), (680.0, 440.0))
                    .build(|| {
                        responses.iter()
                                .for_each(|resp| {
                                    // let title = resp.title.clone();
                                    let mut location = String::from("/");
                                    location.push_str(&resp.location[0].clone().to_str().unwrap()
                                                            .replace(char::from(0), "/"));
                                    // ui.text_colored(TEXT_COLOR, &ImString::from(title));
                                    // ui.same_line(0.0);
                                    if ui.button(&ImString::from(location.clone()), (400.0, ui.get_text_line_height_with_spacing())) {
                                        println!("Trying to open {:?}", location.clone());
                                        Command::new("open")
                                                .arg(location.clone())
                                                .output()
                                                .expect("Failed to open file");
                                    }
                                });
                    });
            }
        });
    });


    RunState {
        status: true,
        showing_results: state.response.is_some()
    }
}

struct Icon {
    texture_id: ImTexture,
    size: (f32, f32),
}

impl Icon {
    fn new<F>(gl_ctx: &F, textures: &mut Textures) -> Result<Self, Box<dyn Error>>
    where
        F: Facade,
    {
        let lenna_bytes = include_bytes!("../../assets/dark_search.jpg");
        let byte_stream = Cursor::new(lenna_bytes.as_ref());
        let decoder = JPEGDecoder::new(byte_stream)?;

        let (width, height) = decoder.dimensions();
        let image = decoder.read_image()?;
        let raw = RawImage2d {
            data: Cow::Owned(image),
            width: width as u32,
            height: height as u32,
            format: ClientFormat::U8U8U8,
        };
        let gl_texture = Texture2d::new(gl_ctx, raw)?;
        let texture_id = textures.insert(gl_texture);
        Ok(Icon {
            texture_id,
            size: (width as f32, height as f32),
        })
    }

    fn show(&self, ui: &Ui) {
        ui.image(self.texture_id, self.size).build();
    }
}