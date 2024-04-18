#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
extern crate alloc;
mod display;
mod leds;
use core::fmt;
use core::mem::MaybeUninit;

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};

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

#[derive(Clone, Copy)]
pub struct DisplayController(
    &'static Mutex<
        CriticalSectionRawMutex,
        Ssd1306<
            I2CInterface<I2C<'static, I2C0>>,
            DisplaySize128x64,
            BufferedGraphicsMode<DisplaySize128x64>,
        >,
    >,
);

pub enum BoardState {
    CNC,
    Roller,
}

impl fmt::Display for BoardState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BoardState::CNC => write!(f, "CNC"),
            BoardState::Roller => write!(f, "Roller"),
        }
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

    let button = io.pins.gpio14.into_pull_down_input();

    let machine_pin = io.pins.gpio12.into_pull_down_input();

    let machine_is_on_led = io.pins.gpio17.into_push_pull_output();
    let machine_is_off_led = io.pins.gpio16.into_push_pull_output();

    let mut system_ready_pin = io.pins.gpio2.into_push_pull_output();

    let interface = I2CDisplayInterface::new(i2c);

    let display = make_static!(Mutex::new(
        Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode()
    ));

    let display_controller = DisplayController(display);

    let machine_state_signal: &'static Signal<CriticalSectionRawMutex, bool> =
        &*make_static!(Signal::new());

    spawner
        .spawn(leds::control_machine_state(
            machine_is_on_led,
            machine_is_off_led,
            machine_state_signal,
        ))
        .ok();

    // Setup default state of Esp32
    display::show_rust_logo(display_controller).await;
    Timer::after(Duration::from_secs(2)).await;
    display::change_board_mode(display_controller, BoardState::CNC).await; //Default mode on start esp32
    machine_state_signal.signal(false); // Default state of led is green (false)

    system_ready_pin.set_high().unwrap();

    let mut board_state: bool = true; // CNC
    let mut old_button_state: bool = false;
    let mut old_machine_state: bool = false;
    loop {
        let button_state = button.is_high().unwrap();
        let machine_is_running = machine_pin.is_high().unwrap();

        // Turn on or off led based on machine state
        if machine_is_running != old_machine_state {
            machine_state_signal.signal(machine_is_running);
            old_machine_state = machine_is_running;
        }

        if button_state != old_button_state && button_state {
            println!("Button pressed!");

            if !machine_is_running {
                board_state = !board_state;
                match board_state {
                    true => display::change_board_mode(display_controller, BoardState::CNC).await,
                    false => {
                        display::change_board_mode(display_controller, BoardState::Roller).await
                    }
                }
            } else {
                display::machine_is_on(display_controller).await;
            }
        };

        old_button_state = button_state;
        Timer::after(Duration::from_millis(100)).await;
    }
}
