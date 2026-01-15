#![no_std]

use embassy_stm32::{
    i2c::{I2c, Master},
    mode::Async,
};
use embassy_time::Timer;

const RS: u8 = 0;
const _RW: u8 = 1;
const EN: u8 = 2;
const BL: u8 = 3;
const CMD_MODE: u8 = 0;
const DATA_MODE: u8 = 1;

pub struct LCD<'a> {
    i2c: I2c<'a, Async, Master>,
    addr: u8,
    back_light: u8,
    display: u8,
    cursor: u8,
    blink: u8,
}

impl<'a> LCD<'a> {
    pub async fn init(i2c: I2c<'a, Async, Master>, addr: u8) -> Self {
        let mut lcd = Self {
            i2c,
            addr,
            back_light: 1,
            display: 1,
            cursor: 1,
            blink: 1,
        };

        Timer::after_millis(50).await;
        lcd.enable_pulse(0x3 << 4).await;
        Timer::after_millis(5).await;
        lcd.enable_pulse(0x3 << 4).await;
        Timer::after_micros(100).await;
        lcd.enable_pulse(0x3 << 4).await;
        lcd.enable_pulse(0x2 << 4).await;

        lcd.function_set().await;
        lcd.clear_display().await;
        lcd.display_control().await;
        lcd.entry_mode_set().await;

        lcd
    }

    pub async fn clear_display(&mut self) {
        const CLEAR_DISPLAY: u8 = 1 << 0;
        self.write(CLEAR_DISPLAY, CMD_MODE).await;
        Timer::after_millis(2).await;
    }

    pub async fn set_cursor_home(&mut self) {
        const SET_HOME: u8 = 1 << 1;
        self.write(SET_HOME, CMD_MODE).await;
        Timer::after_millis(2).await;
    }

    async fn entry_mode_set(&mut self) {
        const ENTRY_MODE_SET: u8 = 1 << 2;
        const ID: u8 = 1 << 1;
        const S: u8 = 0 << 0;
        self.write(ENTRY_MODE_SET | ID | S, CMD_MODE).await;
    }

    async fn display_control(&mut self) {
        const DISPLAY_CONTROL: u8 = 1 << 3;
        const D: u8 = 2;
        const C: u8 = 1;
        const B: u8 = 0;
        self.write(
            DISPLAY_CONTROL | (self.display << D) | (self.cursor << C) | (self.blink << B),
            CMD_MODE,
        )
        .await;
    }

    pub async fn display_on(&mut self, flag: bool) {
        self.display = flag.into();
        self.display_control().await;
    }

    pub async fn cursor_on(&mut self, flag: bool) {
        self.cursor = flag.into();
        self.display_control().await;
    }

    pub async fn blink_on(&mut self, flag: bool) {
        self.blink = flag.into();
        self.display_control().await;
    }

    async fn function_set(&mut self) {
        const FUNCTION_SET: u8 = 1 << 5;
        const DL: u8 = 0 << 4;
        const N: u8 = 1 << 3;
        const F: u8 = 0 << 2;
        self.write(FUNCTION_SET | DL | N | F, CMD_MODE).await;
    }

    pub async fn set_cursor(&mut self, row: u8, col: u8) {
        const SET_DDRAM_ADR: u8 = 1 << 7;
        static ROW_OFFSETS: [u8; 4] = [0x00, 0x40, 0x14, 0x54];
        self.write(SET_DDRAM_ADR | ROW_OFFSETS[row as usize] | col, CMD_MODE)
            .await;
    }

    pub async fn print(&mut self, data: &str) {
        for character in data.as_bytes() {
            self.write(*character, DATA_MODE).await;
        }
    }

    pub async fn back_light_on(&mut self, flag: bool) {
        self.back_light = flag.into();
        self.write(0, CMD_MODE).await;
    }

    async fn write(&mut self, data: u8, mode: u8) {
        let high_bits = (mode << RS) | (self.back_light << BL) | (data & 0xF0);
        self.enable_pulse(high_bits).await;

        let low_bits = (mode << RS) | (self.back_light << BL) | (data << 4);
        self.enable_pulse(low_bits).await;
    }

    async fn enable_pulse(&mut self, data: u8) {
        self.i2c
            .write(self.addr, &[(1 << EN) | data])
            .await
            .unwrap();
        Timer::after_micros(1).await;

        self.i2c
            .write(self.addr, &[(0 << EN) | data])
            .await
            .unwrap();
        Timer::after_micros(50).await;
    }
}
