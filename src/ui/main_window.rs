use glium::{
    backend::Facade,
    texture::{ClientFormat, RawImage2d},
    Texture2d,
};
use image::{jpeg::JPEGDecoder, ImageDecoder};
use imgui::*;

use std::borrow::Cow;
use std::error::Error;
use std::io::Cursor;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender};

use super::support::RunState;
use crate::query_executor::QueryResponse;

const WINDOW_BG: [f32; 4] = [0.27, 0.27, 0.28, 1.0];

struct State {
    query: ImString,
    icon_texture: Option<Icon>,
    response: Option<QueryResponse>,
}

const SEARCH_SIZE: [f32; 2] = [680.0, 50.0];
const RESULTS_SIZE: [f32; 2] = [680.0, 500.0];

pub fn run_window(query_sender: Sender<String>, results_receiver: Receiver<QueryResponse>) {
    let mut state = State {
        query: ImString::with_capacity(128),
        icon_texture: None,
        response: None,
    };
    let mut system = super::support::init("podium");
    if state.icon_texture.is_none() {
        state.icon_texture =
            Some(Icon::new(system.display.get_context(), system.renderer.textures()).unwrap());
    }
    let san_fran_big = system.imgui.fonts().add_font(&[FontSource::TtfData {
        data: include_bytes!("../../assets/System San Francisco Display Regular.ttf"),
        size_pixels: system.font_size * 2.1,
        config: None,
    }]);

    let san_fran = system.imgui.fonts().add_font(&[FontSource::TtfData {
        data: include_bytes!("../../assets/System San Francisco Display Regular.ttf"),
        size_pixels: system.font_size,
        config: None,
    }]);
    system
        .renderer
        .reload_font_texture(&mut system.imgui)
        .expect("Failed to reload fonts");
    system.main_loop(|_, ui| {
        let _sf_big = ui.push_font(san_fran_big);
        hello_world(&mut state, ui, &query_sender, &results_receiver, san_fran)
    });
}

fn hello_world<'a>(
    state: &mut State,
    ui: &Ui<'a>,
    query_sender: &Sender<String>,
    response_receiver: &Receiver<QueryResponse>,
    san_fran: FontId,
) -> RunState {
    let size = if state.response.is_some() {
        RESULTS_SIZE
    } else {
        SEARCH_SIZE
    };

    let _window_bg = ui.push_style_color(StyleColor::WindowBg, WINDOW_BG);

    ui.window(im_str!("Search box"))
        .position([0.0, 0.0], Condition::Always)
        .size(size, Condition::Always)
        .title_bar(false)
        // .inputs(true)
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
            let _input_bg = ui.push_style_color(StyleColor::FrameBg, WINDOW_BG);
            let _input_bg_a = ui.push_style_color(StyleColor::FrameBgActive, WINDOW_BG);

            if ui
                .input_text(im_str!("Search"), &mut state.query)
                .enter_returns_true(true)
                .build()
            {
                ui.set_keyboard_focus_here(FocusedWidget::Next);
                if !state.query.to_str().trim().is_empty() {
                    info!("Searching for {:?}", &state.query.to_str());
                    if query_sender
                        .send(String::from(state.query.to_str()))
                        .is_err()
                    {
                        error!("Failed to send search query");
                    }
                    let resp = response_receiver.recv().unwrap();
                    info!("Found results: {:?}", &resp);
                    state.response = Some(resp);
                } else {
                    state.response = None;
                }
            }

            if let Some(responses) = &state.response {
                ui.separator();
                let _sf = ui.push_font(san_fran);
                ui.child_frame(im_str!("Results box"), [680.0, 440.0])
                    .build(|| {
                        let _btn_style = ui.push_style_var(StyleVar::ButtonTextAlign([0.0, 1.0]));
                        let _input_bg = ui.push_style_color(StyleColor::Button, WINDOW_BG);
                        responses.iter().for_each(|resp| {
                            let location = resp.location[0].to_str().unwrap();
                            if ui.button(
                                &ImString::from(String::from(location)),
                                [400.0, ui.get_text_line_height_with_spacing()],
                            ) {
                                info!("Trying to open {:?}", location);
                                if opener::open(location).is_err() {
                                    error!("Failed to open: {:?}", location);
                                }
                            }
                        });
                    });
            }
        });

    RunState {
        status: true,
        showing_results: state.response.is_some(),
    }
}

struct Icon {
    texture_id: TextureId,
    size: [f32; 2],
}

impl Icon {
    fn new<F>(gl_ctx: &F, textures: &mut Textures<Rc<Texture2d>>) -> Result<Self, Box<dyn Error>>
    where
        F: Facade,
    {
        let lenna_bytes = include_bytes!("../../assets/Search.jpg");
        let byte_stream: Cursor<&[u8]> = Cursor::new(lenna_bytes.as_ref());
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
        let texture_id = textures.insert(Rc::new(gl_texture));
        Ok(Icon {
            texture_id,
            size: [width as f32, height as f32],
        })
    }

    fn show(&self, ui: &Ui) {
        ui.image(self.texture_id, self.size).build();
    }
}
