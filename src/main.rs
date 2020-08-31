#![no_std]
#![no_main]

extern crate tiny;

use panic_halt as _;

use cortex_m::{peripheral::syst::SystClkSource, Peripherals};
use cortex_m_rt::{entry, exception};
use stm32f0xx_hal::{pac, prelude::*};

use tiny::time_source::TimeSource;
use tiny::timer::{Timer, TimerGroup};

use stm32f0xx_hal::gpio::gpioc::PC13;
use stm32f0xx_hal::gpio::{Output, PushPull};

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

struct App<'a> {
    timer_group: &'a TimerGroup<'a>,
    timer: Timer<'a>,
    led: RefCell<PC13<Output<PushPull>>>,
}

impl<'a> App<'a> {
    fn new(timer_group: &'a TimerGroup<'a>, led: PC13<Output<PushPull>>) -> Self {
        Self {
            timer_group,
            timer: TimerGroup::new_timer(),
            led: RefCell::new(led),
        }
    }

    fn start(&'a self) {
        self.arm_timer();
    }

    fn arm_timer(&'a self) {
        self.led.borrow_mut().toggle().ok();
        self.timer_group.start(&self.timer, 500, self, |app| app.arm_timer());
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

    cp.SYST.set_clock_source(SystClkSource::Core);
    cp.SYST.set_reload(8_000_000 / 1000);
    cp.SYST.clear_current();
    cp.SYST.enable_counter();
    cp.SYST.enable_interrupt();

    let timer_group = TimerGroup::new(&time_source);

    let app = App::new(&timer_group, led);
    app.start();

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
