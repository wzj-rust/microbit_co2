use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    watch::{DynReceiver, Watch},
};
use embassy_time::{Delay, Timer};
use libscd::asynchronous::scd4x::Scd4x;
use microbit_bsp::embassy_nrf::{
    Peri, bind_interrupts,
    peripherals::{P0_26, P1_00, TWISPI0},
    twim::{self, Twim},
};
use static_cell::ConstStaticCell;

// CO2消费者数量，分别是 display 和 ble
const CO2_CONSUMERS: usize = 2;
static CO2: Watch<ThreadModeRawMutex, u16, CO2_CONSUMERS> = Watch::new();

// Temperature消费者数量，分别是 ble
const TEMPERATURE_CONSUMERS: usize = 1;
static TEMPERATURE: Watch<ThreadModeRawMutex, i8, TEMPERATURE_CONSUMERS> = Watch::new();

// Humidity消费者数量，分别是 ble
const HUMIDITY_CONSUMERS: usize = 1;
static HUMIDITY: Watch<ThreadModeRawMutex, u8, HUMIDITY_CONSUMERS> = Watch::new();

pub fn get_co2_receiver() -> Option<DynReceiver<'static, u16>> {
    CO2.dyn_receiver()
}

pub fn get_temperature_receiver() -> Option<DynReceiver<'static, i8>> {
    TEMPERATURE.dyn_receiver()
}

pub fn get_humidity_receiver() -> Option<DynReceiver<'static, u8>> {
    HUMIDITY.dyn_receiver()
}

#[embassy_executor::task]
pub async fn sense_task(
    twi: Peri<'static, TWISPI0>,
    sda: Peri<'static, P1_00>,
    scl: Peri<'static, P0_26>,
) {
    bind_interrupts!(struct Irqs {
        TWISPI0 => twim::InterruptHandler<TWISPI0>;
    });
    static RAM_BUFFER: ConstStaticCell<[u8; 4]> = ConstStaticCell::new([0; 4]);
    let i2c = Twim::new(twi, Irqs, sda, scl, Default::default(), RAM_BUFFER.take());

    // 将控制器包装在互斥锁中
    // let i2c_bus = Mutex::new(i2c, Default::default());
    // 为每个驱动程序生成一个 I2C 设备
    // let i2c_dev1 = I2cDevice::new(i2c_bus);

    let mut scd = Scd4x::new(i2c, Delay);

    Timer::after_millis(30).await;

    // When re-programming, the controller will be restarted,
    // but not the sensor. We try to stop it in order to
    // prevent the rest of the commands failing.
    _ = scd.stop_periodic_measurement().await;

    defmt::info!("Sensor serial number: {:?}", scd.serial_number().await);
    if let Err(e) = scd.start_periodic_measurement().await {
        defmt::panic!("Failed to start periodic measurement: {:?}", e);
    }

    let tx_co2 = CO2.sender();
    let tx_temperature = TEMPERATURE.sender();
    let tx_humidity = HUMIDITY.sender();
    loop {
        if scd.data_ready().await.unwrap() {
            let m = scd.read_measurement().await.unwrap();
            defmt::info!(
                "CO2(二氧化碳): {}, Humidity(湿度): {}, Temperature(温度): {}",
                m.co2,
                m.humidity,
                m.temperature
            );
            tx_co2.send(m.co2 as u16);
            tx_temperature.send(m.temperature as i8);
            tx_humidity.send(m.humidity as u8);
        }

        Timer::after_millis(1000).await;
    }
}
