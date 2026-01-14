# An async STM32 LCD I2C library using Embassy framework

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

    let i2c = I2c::new(
        p.I2C1,
        p.PB6,
        p.PB7,
        Irqs,
        p.DMA1_CH6,
        p.DMA1_CH7,
        Default::default(),
    );

    let mut lcd = stm32_lcd_i2c::LCD::init(i2c, ADRESS).await;
    lcd.blink_on(false).await;
    lcd.cursor_on(false).await;
    lcd.set_cursor(1, 2).await;
    lcd.print("hello world").await;

    loop {}
}
```
