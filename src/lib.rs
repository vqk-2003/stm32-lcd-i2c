#![no_std]

use embedded_hal_async::{
    delay::DelayNs,
    i2c::{I2c, SevenBitAddress},
};

const RS: u8 = 0;
const _RW: u8 = 1;
const EN: u8 = 2;
const BL: u8 = 3;
const CMD_MODE: u8 = 0;
const DATA_MODE: u8 = 1;

pub struct LCD<I2C, DELAY> {
    i2c: I2C,
    delay: DELAY,
    addr: u8,
    back_light: bool,
    display: bool,
    cursor: bool,
    blink: bool,
}

impl<I2C: I2c<SevenBitAddress>, DELAY: DelayNs> LCD<I2C, DELAY> {
    pub fn new(i2c: I2C, addr: u8, delay: DELAY) -> Self {
        Self {
            i2c,
            delay,
            addr,
            back_light: true,
            display: true,
            cursor: true,
            blink: true,
        }
    }

    pub async fn init(&mut self) -> Result<(), I2C::Error> {
        self.delay.delay_ms(50).await;
        self.enable_pulse(0x3 << 4).await?;
        self.delay.delay_ms(5).await;
        self.enable_pulse(0x3 << 4).await?;
        self.delay.delay_ms(100).await;
        self.enable_pulse(0x3 << 4).await?;
        self.enable_pulse(0x2 << 4).await?;

        self.function_set().await?;
        self.clear_display().await?;
        self.display_control().await?;
        self.entry_mode_set().await?;

        Ok(())
    }

    pub async fn clear_display(&mut self) -> Result<(), I2C::Error> {
        const CLEAR_DISPLAY: u8 = 1 << 0;
        self.write(CLEAR_DISPLAY, CMD_MODE).await?;
        self.delay.delay_ms(2).await;
        Ok(())
    }

    pub async fn set_cursor_home(&mut self) -> Result<(), I2C::Error> {
        const SET_HOME: u8 = 1 << 1;
        self.write(SET_HOME, CMD_MODE).await?;
        self.delay.delay_ms(2).await;
        Ok(())
    }

    async fn entry_mode_set(&mut self) -> Result<(), I2C::Error> {
        const ENTRY_MODE_SET: u8 = 1 << 2;
        const ID: u8 = 1 << 1;
        const S: u8 = 0;
        self.write(ENTRY_MODE_SET | ID | S, CMD_MODE).await?;
        Ok(())
    }

    async fn display_control(&mut self) -> Result<(), I2C::Error> {
        const DISPLAY_CONTROL: u8 = 1 << 3;
        const D: u8 = 2;
        const C: u8 = 1;
        const B: u8 = 0;
        self.write(
            DISPLAY_CONTROL
                | (u8::from(self.display) << D)
                | (u8::from(self.cursor) << C)
                | (u8::from(self.blink) << B),
            CMD_MODE,
        )
        .await?;
        Ok(())
    }

    pub async fn display_on(&mut self, enabled: bool) -> Result<(), I2C::Error> {
        self.display = enabled;
        self.display_control().await?;
        Ok(())
    }

    pub async fn cursor_on(&mut self, enabled: bool) -> Result<(), I2C::Error> {
        self.cursor = enabled;
        self.display_control().await?;
        Ok(())
    }

    pub async fn blink_on(&mut self, enabled: bool) -> Result<(), I2C::Error> {
        self.blink = enabled;
        self.display_control().await?;
        Ok(())
    }

    async fn function_set(&mut self) -> Result<(), I2C::Error> {
        const FUNCTION_SET: u8 = 1 << 5;
        const DL: u8 = 0 << 4;
        const N: u8 = 1 << 3;
        const F: u8 = 0 << 2;
        self.write(FUNCTION_SET | DL | N | F, CMD_MODE).await?;
        Ok(())
    }

    pub async fn set_cursor(&mut self, row: u8, col: u8) -> Result<(), I2C::Error> {
        const SET_DDRAM_ADR: u8 = 1 << 7;
        static ROW_OFFSETS: [u8; 4] = [0x00, 0x40, 0x14, 0x54];
        self.write(SET_DDRAM_ADR | ROW_OFFSETS[row as usize] | col, CMD_MODE)
            .await?;
        Ok(())
    }

    pub async fn print(&mut self, data: &str) -> Result<(), I2C::Error> {
        for character in data.as_bytes() {
            self.write(*character, DATA_MODE).await?;
        }
        Ok(())
    }

    pub async fn back_light_on(&mut self, enabled: bool) -> Result<(), I2C::Error> {
        self.back_light = enabled;
        self.write(0, CMD_MODE).await?;
        Ok(())
    }

    async fn write(&mut self, data: u8, mode: u8) -> Result<(), I2C::Error> {
        let high_bits = (mode << RS) | (u8::from(self.back_light) << BL) | (data & 0xF0);
        self.enable_pulse(high_bits).await?;

        let low_bits = (mode << RS) | (u8::from(self.back_light) << BL) | (data << 4);
        self.enable_pulse(low_bits).await?;

        Ok(())
    }

    async fn enable_pulse(&mut self, data: u8) -> Result<(), I2C::Error> {
        self.i2c.write(self.addr, &[(1 << EN) | data]).await?;
        self.delay.delay_us(1).await;

        self.i2c.write(self.addr, &[(0 << EN) | data]).await?;
        self.delay.delay_us(500).await;

        Ok(())
    }
}
