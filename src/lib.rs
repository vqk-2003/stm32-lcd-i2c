#![no_std]

use embedded_hal_async::{delay::DelayNs, i2c::I2c};

const RS: u8 = 0;
const _RW: u8 = 1;
const EN: u8 = 2;
const BL: u8 = 3;
const CMD_MODE: u8 = 0;
const DATA_MODE: u8 = 1;

pub struct LCD {
    addr: u8,
    back_light: u8,
    display: u8,
    cursor: u8,
    blink: u8,
}

impl LCD {
    pub async fn init<BUS, DELAY>(
        addr: u8,
        i2c: &mut BUS,
        delay: &mut DELAY,
    ) -> Result<Self, BUS::Error>
    where
        BUS: I2c,
        DELAY: DelayNs,
    {
        let mut lcd = Self {
            addr,
            back_light: 1,
            display: 1,
            cursor: 1,
            blink: 1,
        };

        delay.delay_ms(50).await;
        lcd.enable_pulse(0x3 << 4, i2c, delay).await?;
        delay.delay_ms(5).await;
        lcd.enable_pulse(0x3 << 4, i2c, delay).await?;
        delay.delay_ms(100).await;
        lcd.enable_pulse(0x3 << 4, i2c, delay).await?;
        lcd.enable_pulse(0x2 << 4, i2c, delay).await?;

        lcd.function_set(i2c, delay).await?;
        lcd.clear_display(i2c, delay).await?;
        lcd.display_control(i2c, delay).await?;
        lcd.entry_mode_set(i2c, delay).await?;

        Ok(lcd)
    }

    pub async fn clear_display<BUS, DELAY>(
        &mut self,
        i2c: &mut BUS,
        delay: &mut DELAY,
    ) -> Result<(), BUS::Error>
    where
        BUS: I2c,
        DELAY: DelayNs,
    {
        const CLEAR_DISPLAY: u8 = 1 << 0;
        self.write(CLEAR_DISPLAY, CMD_MODE, i2c, delay).await?;
        delay.delay_ms(2).await;
        Ok(())
    }

    pub async fn set_cursor_home<BUS, DELAY>(
        &mut self,
        i2c: &mut BUS,
        delay: &mut DELAY,
    ) -> Result<(), BUS::Error>
    where
        BUS: I2c,
        DELAY: DelayNs,
    {
        const SET_HOME: u8 = 1 << 1;
        self.write(SET_HOME, CMD_MODE, i2c, delay).await?;
        delay.delay_ms(2).await;
        Ok(())
    }

    async fn entry_mode_set<BUS, DELAY>(
        &mut self,
        i2c: &mut BUS,
        delay: &mut DELAY,
    ) -> Result<(), BUS::Error>
    where
        BUS: I2c,
        DELAY: DelayNs,
    {
        const ENTRY_MODE_SET: u8 = 1 << 2;
        const ID: u8 = 1 << 1;
        const S: u8 = 0;
        self.write(ENTRY_MODE_SET | ID | S, CMD_MODE, i2c, delay)
            .await?;
        Ok(())
    }

    async fn display_control<BUS, DELAY>(
        &mut self,
        i2c: &mut BUS,
        delay: &mut DELAY,
    ) -> Result<(), BUS::Error>
    where
        BUS: I2c,
        DELAY: DelayNs,
    {
        const DISPLAY_CONTROL: u8 = 1 << 3;
        const D: u8 = 2;
        const C: u8 = 1;
        const B: u8 = 0;
        self.write(
            DISPLAY_CONTROL | (self.display << D) | (self.cursor << C) | (self.blink << B),
            CMD_MODE,
            i2c,
            delay,
        )
        .await?;
        Ok(())
    }

    pub async fn display_on<BUS, DELAY>(
        &mut self,
        flag: bool,
        i2c: &mut BUS,
        delay: &mut DELAY,
    ) -> Result<(), BUS::Error>
    where
        BUS: I2c,
        DELAY: DelayNs,
    {
        self.display = flag.into();
        self.display_control(i2c, delay).await?;
        Ok(())
    }

    pub async fn cursor_on<BUS, DELAY>(
        &mut self,
        flag: bool,
        i2c: &mut BUS,
        delay: &mut DELAY,
    ) -> Result<(), BUS::Error>
    where
        BUS: I2c,
        DELAY: DelayNs,
    {
        self.cursor = flag.into();
        self.display_control(i2c, delay).await?;
        Ok(())
    }

    pub async fn blink_on<BUS, DELAY>(
        &mut self,
        flag: bool,
        i2c: &mut BUS,
        delay: &mut DELAY,
    ) -> Result<(), BUS::Error>
    where
        BUS: I2c,
        DELAY: DelayNs,
    {
        self.blink = flag.into();
        self.display_control(i2c, delay).await?;
        Ok(())
    }

    async fn function_set<BUS, DELAY>(
        &mut self,
        i2c: &mut BUS,
        delay: &mut DELAY,
    ) -> Result<(), BUS::Error>
    where
        BUS: I2c,
        DELAY: DelayNs,
    {
        const FUNCTION_SET: u8 = 1 << 5;
        const DL: u8 = 0 << 4;
        const N: u8 = 1 << 3;
        const F: u8 = 0 << 2;
        self.write(FUNCTION_SET | DL | N | F, CMD_MODE, i2c, delay)
            .await?;
        Ok(())
    }

    pub async fn set_cursor<BUS, DELAY>(
        &mut self,
        row: u8,
        col: u8,
        i2c: &mut BUS,
        delay: &mut DELAY,
    ) -> Result<(), BUS::Error>
    where
        BUS: I2c,
        DELAY: DelayNs,
    {
        const SET_DDRAM_ADR: u8 = 1 << 7;
        static ROW_OFFSETS: [u8; 4] = [0x00, 0x40, 0x14, 0x54];
        self.write(
            SET_DDRAM_ADR | ROW_OFFSETS[row as usize] | col,
            CMD_MODE,
            i2c,
            delay,
        )
        .await?;
        Ok(())
    }

    pub async fn print<BUS, DELAY>(
        &mut self,
        data: &str,
        i2c: &mut BUS,
        delay: &mut DELAY,
    ) -> Result<(), BUS::Error>
    where
        BUS: I2c,
        DELAY: DelayNs,
    {
        for character in data.as_bytes() {
            self.write(*character, DATA_MODE, i2c, delay).await?;
        }
        Ok(())
    }

    pub async fn back_light_on<BUS, DELAY>(
        &mut self,
        flag: bool,
        i2c: &mut BUS,
        delay: &mut DELAY,
    ) -> Result<(), BUS::Error>
    where
        BUS: I2c,
        DELAY: DelayNs,
    {
        self.back_light = flag.into();
        self.write(0, CMD_MODE, i2c, delay).await?;
        Ok(())
    }

    async fn write<BUS, DELAY>(
        &mut self,
        data: u8,
        mode: u8,
        i2c: &mut BUS,
        delay: &mut DELAY,
    ) -> Result<(), BUS::Error>
    where
        BUS: I2c,
        DELAY: DelayNs,
    {
        let high_bits = (mode << RS) | (self.back_light << BL) | (data & 0xF0);
        self.enable_pulse(high_bits, i2c, delay).await?;

        let low_bits = (mode << RS) | (self.back_light << BL) | (data << 4);
        self.enable_pulse(low_bits, i2c, delay).await?;

        Ok(())
    }

    async fn enable_pulse<BUS, DELAY>(
        &mut self,
        data: u8,
        i2c: &mut BUS,
        delay: &mut DELAY,
    ) -> Result<(), BUS::Error>
    where
        BUS: I2c,
        DELAY: DelayNs,
    {
        i2c.write(self.addr, &[(1 << EN) | data]).await?;
        delay.delay_us(1).await;

        i2c.write(self.addr, &[(0 << EN) | data]).await?;
        delay.delay_us(500).await;

        Ok(())
    }
}
