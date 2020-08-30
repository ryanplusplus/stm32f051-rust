#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m::{peripheral::syst::SystClkSource, Peripherals};
use cortex_m_rt::{entry, exception};
use stm32f0xx_hal::{pac, prelude::*};

use stm32f051_rust::time_source::TimeSource;
use stm32f051_rust::timer::TimerGroup;

use core::cell::RefCell;

static mut TICKS: u32 = 0;

struct SysTickTimeSource {}

impl SysTickTimeSource {
    fn new() -> Self {
        Self {}
    }
}

impl TimeSource for SysTickTimeSource {
    fn ticks(&self) -> u32 {
        unsafe { TICKS }
    }
}

#[entry]
fn main() -> ! {
    let time_source = SysTickTimeSource::new();

    let mut p = pac::Peripherals::take().unwrap();
    let mut cp = Peripherals::take().unwrap();
    let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut p.FLASH);

    let gpioc = p.GPIOC.split(&mut rcc);
    let led = cortex_m::interrupt::free(move |cs| gpioc.pc13.into_push_pull_output(cs));

    let led = RefCell::new(led);

    cp.SYST.set_clock_source(SystClkSource::Core);
    cp.SYST.set_reload(8_000_000 / 1000);
    cp.SYST.clear_current();
    cp.SYST.enable_counter();
    cp.SYST.enable_interrupt();

    let mut timer_group = TimerGroup::new(&time_source);
    let timer = TimerGroup::new_timer();

    timer_group.start(&timer, 500, &led, |led| {
        led.borrow_mut().toggle().ok();
    });

    loop {
        cortex_m::asm::wfi();
        timer_group.run();
    }
}

#[exception]
fn SysTick() {
    unsafe {
        TICKS += 1;
    }
}
