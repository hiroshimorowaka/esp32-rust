#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
extern crate alloc;
use alloc::format;
use core::mem::MaybeUninit;
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};
use embedded_graphics::image::{Image, ImageRaw};

use esp_backtrace as _;
use esp_hal::i2c::I2C;
use esp_hal::macros::main;
use esp_hal::peripherals::I2C0;
use esp_hal::prelude::{_embedded_hal_digital_v2_InputPin, _embedded_hal_digital_v2_OutputPin};
use esp_hal::system::SystemExt;
use esp_hal::timer::TimerGroup;
use esp_hal::IO;
use esp_hal::{
    clock::ClockControl, embassy, entry, peripherals::Peripherals, prelude::_fugit_RateExtU32,
};
use esp_println::println;

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use static_cell::make_static;
#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
    }
}
#[main]
async fn main(spawner: Spawner) {
    init_heap();

    println!("Init!");

    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::max(system.clock_control).freeze();

    let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timg0);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let i2c = I2C::new(
        peripherals.I2C0,
        io.pins.gpio21,
        io.pins.gpio22,
        100.kHz(),
        &clocks,
    );

    let interface = I2CDisplayInterface::new(i2c);

    let display =
        make_static!(
            Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
                .into_buffered_graphics_mode()
        );

    let board_state_signal: &'static Signal<CriticalSectionRawMutex, bool> =
        &*make_static!(Signal::new());

    let machine_state_signal: &'static Signal<CriticalSectionRawMutex, bool> =
        &*make_static!(Signal::new());

    let button = io.pins.gpio14.into_pull_down_input();

    let machine_pin = io.pins.gpio12.into_pull_down_input();

    let mut system_ready_pin = io.pins.gpio2.into_push_pull_output();

    spawner
        .spawn(graphics(display, board_state_signal, machine_state_signal))
        .ok();

    system_ready_pin.set_high().unwrap();

    let mut old_button_state: bool = false;
    let mut board_state: bool = false;

    loop {
        let button_state = button.is_high().unwrap();
        let machine_is_running = machine_pin.is_high().unwrap();

        if button_state != old_button_state && button_state {
            println!("Button pressed!");

            if !machine_is_running {
                board_state = !board_state;

                board_state_signal.signal(board_state);
            } else {
                machine_state_signal.signal(true);
            }
        };

        old_button_state = button_state;
        Timer::after(Duration::from_millis(100)).await;
    }
}

#[embassy_executor::task]
async fn graphics(
    display: &'static mut Ssd1306<
        I2CInterface<I2C<'static, I2C0>>,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>,
    >,
    control_board: &'static Signal<CriticalSectionRawMutex, bool>,
    control_machine: &'static Signal<CriticalSectionRawMutex, bool>,
) {
    display.init().unwrap();

    let raw: ImageRaw<BinaryColor> = ImageRaw::new(include_bytes!("./rust.raw"), 64);

    let im = Image::new(&raw, Point::new(32, 0));

    im.draw(display).unwrap();

    display.flush().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let mut actual_machine_state = false; // CNC

    loop {
        if control_machine.signaled() {
            // Check if machine is running (cannot change mode while machine is running)
            display.clear(BinaryColor::Off).unwrap();

            Text::with_baseline(
                format!("Desligue a máquina!").as_str(),
                Point::new(32, 32), // Centered text
                text_style,
                Baseline::Top,
            )
            .draw(display)
            .unwrap();

            display.flush().unwrap();
        }

        if control_board.signaled() && !control_machine.signaled() {
            actual_machine_state = !actual_machine_state;

            let mode_string = match actual_machine_state {
                true => "Roller",
                false => "CNC",
            };

            display.clear(BinaryColor::Off).unwrap();

            Text::with_baseline(
                format!("Modo: {mode_string}!").as_str(),
                Point::new(32, 32),
                text_style,
                Baseline::Top,
            )
            .draw(display)
            .unwrap();

            display.flush().unwrap();
        }
        control_machine.reset();
        control_board.reset();
        Timer::after(Duration::from_millis(300)).await;
    }
}
