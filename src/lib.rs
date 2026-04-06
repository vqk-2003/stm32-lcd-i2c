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

pub struct LCD<BUS> {
    bus: BUS,
    addr: u8,
    back_light: bool,
    display: bool,
    cursor: bool,
    blink: bool,
}

impl<BUS: I2c<SevenBitAddress>> LCD<BUS> {
    pub fn new(bus: BUS, addr: u8) -> Self {
        Self {
            bus,
            addr,
            back_light: true,
            display: true,
            cursor: true,
            blink: true,
        }
    }

    pub async fn init<DELAY: DelayNs>(&mut self, delay: &mut DELAY) -> Result<(), BUS::Error> {
        delay.delay_ms(50).await;
        self.enable_pulse(0x3 << 4, delay).await?;
        delay.delay_ms(5).await;
        self.enable_pulse(0x3 << 4, delay).await?;
        delay.delay_ms(100).await;
        self.enable_pulse(0x3 << 4, delay).await?;
        self.enable_pulse(0x2 << 4, delay).await?;

        self.function_set(delay).await?;
        self.clear_display(delay).await?;
        self.display_control(delay).await?;
        self.entry_mode_set(delay).await?;

        Ok(())
    }

    pub async fn clear_display<DELAY: DelayNs>(
        &mut self,
        delay: &mut DELAY,
    ) -> Result<(), BUS::Error> {
        const CLEAR_DISPLAY: u8 = 1 << 0;
        self.write(CLEAR_DISPLAY, CMD_MODE, delay).await?;
        delay.delay_ms(2).await;
        Ok(())
    }

    pub async fn set_cursor_home<DELAY: DelayNs>(
        &mut self,
        delay: &mut DELAY,
    ) -> Result<(), BUS::Error> {
        const SET_HOME: u8 = 1 << 1;
        self.write(SET_HOME, CMD_MODE, delay).await?;
        delay.delay_ms(2).await;
        Ok(())
    }

    async fn entry_mode_set<DELAY: DelayNs>(
        &mut self,
        delay: &mut DELAY,
    ) -> Result<(), BUS::Error> {
        const ENTRY_MODE_SET: u8 = 1 << 2;
        const ID: u8 = 1 << 1;
        const S: u8 = 0;
        self.write(ENTRY_MODE_SET | ID | S, CMD_MODE, delay).await?;
        Ok(())
    }

    async fn display_control<DELAY: DelayNs>(
        &mut self,
        delay: &mut DELAY,
    ) -> Result<(), BUS::Error> {
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
            delay,
        )
        .await?;
        Ok(())
    }

    pub async fn display_on<DELAY: DelayNs>(
        &mut self,
        enabled: bool,
        delay: &mut DELAY,
    ) -> Result<(), BUS::Error> {
        self.display = enabled;
        self.display_control(delay).await?;
        Ok(())
    }

    pub async fn cursor_on<DELAY: DelayNs>(
        &mut self,
        enabled: bool,
        delay: &mut DELAY,
    ) -> Result<(), BUS::Error> {
        self.cursor = enabled;
        self.display_control(delay).await?;
        Ok(())
    }

    pub async fn blink_on<DELAY: DelayNs>(
        &mut self,
        enabled: bool,
        delay: &mut DELAY,
    ) -> Result<(), BUS::Error> {
        self.blink = enabled;
        self.display_control(delay).await?;
        Ok(())
    }

    async fn function_set<DELAY: DelayNs>(&mut self, delay: &mut DELAY) -> Result<(), BUS::Error> {
        const FUNCTION_SET: u8 = 1 << 5;
        const DL: u8 = 0 << 4;
        const N: u8 = 1 << 3;
        const F: u8 = 0 << 2;
        self.write(FUNCTION_SET | DL | N | F, CMD_MODE, delay)
            .await?;
        Ok(())
    }

    pub async fn set_cursor<DELAY: DelayNs>(
        &mut self,
        row: u8,
        col: u8,
        delay: &mut DELAY,
    ) -> Result<(), BUS::Error> {
        const SET_DDRAM_ADR: u8 = 1 << 7;
        static ROW_OFFSETS: [u8; 4] = [0x00, 0x40, 0x14, 0x54];
        self.write(
            SET_DDRAM_ADR | ROW_OFFSETS[row as usize] | col,
            CMD_MODE,
            delay,
        )
        .await?;
        Ok(())
    }

    pub async fn print<DELAY: DelayNs>(
        &mut self,
        data: &str,
        delay: &mut DELAY,
    ) -> Result<(), BUS::Error> {
        for character in data.as_bytes() {
            self.write(*character, DATA_MODE, delay).await?;
        }
        Ok(())
    }

    pub async fn back_light_on<DELAY: DelayNs>(
        &mut self,
        enabled: bool,
        delay: &mut DELAY,
    ) -> Result<(), BUS::Error> {
        self.back_light = enabled;
        self.write(0, CMD_MODE, delay).await?;
        Ok(())
    }

    async fn write<DELAY: DelayNs>(
        &mut self,
        data: u8,
        mode: u8,
        delay: &mut DELAY,
    ) -> Result<(), BUS::Error> {
        let high_bits = (mode << RS) | (u8::from(self.back_light) << BL) | (data & 0xF0);
        self.enable_pulse(high_bits, delay).await?;

        let low_bits = (mode << RS) | (u8::from(self.back_light) << BL) | (data << 4);
        self.enable_pulse(low_bits, delay).await?;

        Ok(())
    }

    async fn enable_pulse<DELAY: DelayNs>(
        &mut self,
        data: u8,
        delay: &mut DELAY,
    ) -> Result<(), BUS::Error> {
        self.bus.write(self.addr, &[(1 << EN) | data]).await?;
        delay.delay_us(1).await;

        self.bus.write(self.addr, &[(0 << EN) | data]).await?;
        delay.delay_us(500).await;

        Ok(())
    }
}
