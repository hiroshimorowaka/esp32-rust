use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use esp_hal::{
    gpio::{GpioPin, Output, PushPull},
    prelude::_embedded_hal_digital_v2_OutputPin,
};

#[embassy_executor::task]
pub async fn control_machine_state(
    mut machine_on: GpioPin<Output<PushPull>, 17>,
    mut machine_off: GpioPin<Output<PushPull>, 16>,
    control: &'static Signal<CriticalSectionRawMutex, bool>,
) {
    loop {
        if control.wait().await {
            machine_on.set_high().unwrap();
            machine_off.set_low().unwrap();
        } else {
            machine_on.set_low().unwrap();
            machine_off.set_high().unwrap();
        }
    }
}
