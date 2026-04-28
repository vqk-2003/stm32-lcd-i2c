# An async LCD I2C library using embedded-hal-async traits

NOTE: This library was first written to rely only Embassy framework but since then it has been migrated to use embedded-hal-async traits, allowing it to be used by more async frameworks.

## Usage
```rust
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    i2c::{self, I2c},
    peripherals,
};
use {defmt_rtt as _, panic_probe as _};

const ADRESS: u8 = 0x27;

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let bus = I2c::new(
        p.I2C1,
        p.PB6,
        p.PB7,
        Irqs,
        p.DMA1_CH6,
        p.DMA1_CH7,
        Default::default(),
    );

    let delay = embassy_time::Delay;

    let mut lcd = stm32_lcd_i2c::LCD::new(bus, ADRESS, delay.clone());
    lcd.init().await.unwrap();
    lcd.blink_on(false).await.unwrap();
    lcd.cursor_on(false).await.unwrap();
    lcd.set_cursor(1, 2).await.unwrap();
    lcd.print("hello khanh").await.unwrap();

    loop {}
}
```
