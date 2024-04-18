use crate::{BoardState, DisplayController};
use alloc::format;
use core::ops::DerefMut;
use embedded_graphics::image::{Image, ImageRaw};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::prelude::*;

pub async fn change_board_mode(display_controller: DisplayController, board_state: BoardState) {
    let mut dsp = display_controller.0.lock().await;
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    dsp.init().unwrap();

    Text::with_baseline(
        format!("Modo: {board_state}!").as_str(),
        Point::new(32, 32), // Centered text
        text_style,
        Baseline::Top,
    )
    .draw(dsp.deref_mut())
    .unwrap();

    dsp.flush().unwrap();
}

pub async fn show_rust_logo(display_controller: DisplayController) {
    let mut dsp = display_controller.0.lock().await;

    dsp.init().unwrap();

    let raw: ImageRaw<BinaryColor> = ImageRaw::new(include_bytes!("./rust.raw"), 64);

    let im = Image::new(&raw, Point::new(32, 0));

    im.draw(dsp.deref_mut()).unwrap();

    dsp.flush().unwrap();
}

pub async fn machine_is_on(display_controller: DisplayController) {
    let mut dsp = display_controller.0.lock().await;
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    dsp.init().unwrap();

    Text::with_baseline(
        format!("Desligue a maquina!").as_str(),
        Point::new(10, 32), // Centered text
        text_style,
        Baseline::Top,
    )
    .draw(dsp.deref_mut())
    .unwrap();

    Text::with_baseline(
        format!("E tente novamente!").as_str(),
        Point::new(10, 42), // Centered text
        text_style,
        Baseline::Top,
    )
    .draw(dsp.deref_mut())
    .unwrap();

    dsp.flush().unwrap();
}
