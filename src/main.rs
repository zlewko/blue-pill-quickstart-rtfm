#![no_main]
#![no_std]

// set the panic handler
extern crate panic_semihosting;

extern crate stm32f1xx_hal as hal;
extern crate stm32f1;
use rtfm::app;

use hal::prelude::*;
use hal::timer::{self, Timer};

// CONSTANTS
const FREQ: u32 = 512;

#[app(device = stm32f1::stm32f103)]
const APP: () = {
    // resources
    static mut LED : stm32f1xx_hal::gpio::gpioc::PC13<stm32f1xx_hal::gpio::Output<stm32f1xx_hal::gpio::PushPull>> = ();
    static mut CNT : u32 = 0;

    #[init ]
    fn init() {
        let mut flash = device.FLASH.constrain();
        let mut rcc = device.RCC.constrain();

        // configure the clocks
        let clocks = rcc.cfgr
            .use_hse(8.mhz())
            .sysclk(64.mhz())
            .pclk1(32.mhz())
            .freeze(&mut flash.acr);
        // configure SysTick to generate 512 exceptions every second
        Timer::syst(core.SYST, FREQ.hz(), clocks).listen(timer::Event::Update);

        // configure the user led
        let mut gpioc = device.GPIOC.split(&mut rcc.apb2);
        let led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

        // late resource assignments
        LED = led;
    }

    #[idle (resources=[LED,CNT])]
    fn idle() -> ! {
        let mut toggle_led : bool = false;;   
        loop {
            cortex_m::asm::wfi();
            // lock and check cnt, want to toggle_led once per second
            resources.CNT.lock(|cnt| {
                if *cnt == FREQ {
                    toggle_led = true;
                    *cnt =  0;
                }
            });
            if toggle_led {
                toggle_led = false;
                resources.LED.toggle();
            }
        }
    }

    #[exception (resources=[CNT])]
    fn SysTick() {
        *resources.CNT += 1;
    }
};