use embassy_time::{Duration, Timer};

use esp_hal::{
    gpio::{GpioPin, Input, Output, PullDown, PushPull},
    prelude::{_embedded_hal_digital_v2_InputPin, _embedded_hal_digital_v2_OutputPin},
};

#[embassy_executor::task]
pub async fn control_machine_state(
    mut machine_on: GpioPin<Output<PushPull>, 17>,
    mut machine_off: GpioPin<Output<PushPull>, 16>,
    machine_pin: &'static GpioPin<Input<PullDown>, 12>,
) {
    let mut old_machine_state: bool = false;

    let initial_state: bool = machine_pin.is_high().unwrap();

    //Set Initial State
    machine_on.set_state(<bool>::into(initial_state)).unwrap();
    machine_off.set_state(<bool>::into(!initial_state)).unwrap();

    loop {
        let machine_is_running = machine_pin.is_high().unwrap();

        if machine_is_running != old_machine_state {
            machine_on
                .set_state(<bool>::into(machine_is_running))
                .unwrap();
            machine_off
                .set_state(<bool>::into(!machine_is_running))
                .unwrap();

            old_machine_state = machine_is_running;
        }

        Timer::after(Duration::from_millis(100)).await;
    }
}
