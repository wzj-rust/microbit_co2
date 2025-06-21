#![no_std]
#![no_main]

mod display;
mod sense;

use defmt_rtt as _;
use embassy_executor::Spawner;
use microbit_bsp::Microbit;
use panic_probe as _;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    defmt::info!("Starting...");
    // let p = embassy_nrf::init(Default::default());
    let b = Microbit::default();
    spawner.must_spawn(sense::sense_task(b.twispi0, b.p20, b.p19));
    spawner.must_spawn(display::display_task(b.display));
}
