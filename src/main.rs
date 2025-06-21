#![no_std]
#![no_main]

mod sense;
mod display;

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use microbit_bsp::Microbit;
use panic_probe as _;
use sense::sense_task;
use display::display_task;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Starting...");
    // let p = embassy_nrf::init(Default::default());
    let b = Microbit::default();
    spawner.must_spawn(sense_task(b.twispi0, b.p20, b.p19));
    spawner.must_spawn(display_task(b.display));
}
