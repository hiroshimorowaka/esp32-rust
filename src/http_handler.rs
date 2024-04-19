use core::ops::DerefMut;

use embassy_time::Duration;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::text::{Baseline, Text};
use embedded_graphics::Drawable;
use embedded_graphics::{
    geometry::Point,
    image::{Image, ImageRaw},
    pixelcolor::BinaryColor,
};

use esp_println::println;
use esp_wifi::wifi::{WifiDevice, WifiStaDevice};
use picoserve::{extract::State, response::IntoResponse, routing::get, Router};
use ssd1306::mode::DisplayConfig;

use crate::{AppState, DisplayController};

#[embassy_executor::task]
pub async fn web_task(
    config: &'static picoserve::Config<Duration>,
    stack: &'static embassy_net::Stack<WifiDevice<'static, WifiStaDevice>>,
    state: AppState,
) {
    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; 1024];

    loop {
        let mut socket = embassy_net::tcp::TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        println!("Listening on TCP:80...");
        if let Err(e) = socket.accept(80).await {
            log::warn!("accept error: {:?}", e);
            continue;
        }

        println!("Received connection from {:?}", socket.remote_endpoint());
        let app = Router::new()
            .route(
                "/",
                get(|| picoserve::response::File::html(include_str!("index.html"))),
            )
            .route(
                "/index.css",
                get(|| picoserve::response::File::css(include_str!("index.css"))),
            )
            .route(
                "/index.js",
                get(|| picoserve::response::File::javascript(include_str!("index.js"))),
            )
            .route("/root", get(getroot))
            .route("/rust", get(show_rust_logo))
            .route("/mode", get(hello_word));
        match picoserve::serve_with_state(&app, &config, &mut [0; 2048], socket, &state).await {
            Ok(_) => {}
            Err(err) => log::error!("Web task error: {err:?}"),
        }
    }
}

async fn show_rust_logo(State(display): State<DisplayController>) -> impl IntoResponse {
    let mut dsp = display.0.lock().await;
    dsp.init().unwrap();
    let raw: ImageRaw<BinaryColor> = ImageRaw::new(include_bytes!("./rust.raw"), 64);

    let im = Image::new(&raw, Point::new(32, 0));

    im.draw(dsp.deref_mut()).unwrap();

    dsp.flush().unwrap();
    "Rust appeared!"
}

async fn hello_word(State(display): State<DisplayController>) -> impl IntoResponse {
    let mut dsp = display.0.lock().await;
    dsp.init().unwrap();
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("Hello world!", Point::zero(), text_style, Baseline::Top)
        .draw(dsp.deref_mut())
        .unwrap();

    Text::with_baseline("Hello Rust!", Point::new(0, 16), text_style, Baseline::Top)
        .draw(dsp.deref_mut())
        .unwrap();

    dsp.flush().unwrap();

    dsp.flush().unwrap();
}

async fn getroot() -> impl IntoResponse {
    "Hello, World from root!"
}
