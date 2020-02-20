#![no_std]
#![no_main]

extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate panic_halt;
extern crate stm32f0xx_hal as hal;

use cortex_m_rt::entry;
use cortex_m::interrupt::{Mutex, free};
use cortex_m::peripheral::Peripherals as c_m_Peripherals;

use ssd1306::{prelude::*, Builder as SSD1306Builder};

use core::fmt;
use core::fmt::Write;
use arrayvec::ArrayString;

use crate::hal::{
    gpio::*,
    prelude::*,
    i2c::I2c,
    delay::Delay,
    stm32::{interrupt, Interrupt, Peripherals, TIM3},
    time::Hertz,
    timers::*,
};

use core::ops::DerefMut;
use core::cell::{Cell, RefCell};


static GINT: Mutex<RefCell<Option<Timer<TIM3>>>> = Mutex::new(RefCell::new(None));

static COUNTER: Mutex<Cell<u8>> = Mutex::new(Cell::new(0u8));


#[interrupt]

fn TIM3() {

    cortex_m::interrupt::free(|cs| COUNTER.borrow(cs).set(COUNTER.borrow(cs).get() + 1));

}


#[entry]
fn main() -> ! {

    if let (Some(mut p), Some(cp)) = (Peripherals::take(), c_m_Peripherals::take()) {
        
        cortex_m::interrupt::free(move |cs| {

        let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut p.FLASH);
        
        let gpioa = p.GPIOA.split(&mut rcc);
                

        let scl = gpioa.pa9.into_alternate_af4(cs);
        let sda = gpioa.pa10.into_alternate_af4(cs);
        
        let i2c = I2c::i2c1(p.I2C1, (scl, sda), 400.khz(), &mut rcc);
        

        // Get delay provider
        let mut delay = Delay::new(cp.SYST, &rcc);

        // Set up a timer expiring after 1s
        let mut timer = Timer::tim3(p.TIM3, Hertz(1), &mut rcc);

        // Generate an interrupt when the timer expires
        timer.listen(Event::TimeOut);

        // Move the timer into our global storage
        *GINT.borrow(cs).borrow_mut() = Some(timer);

        // Enable TIM3 IRQ, set prio 1 and clear any pending IRQs

        let mut nvic = cp.NVIC;
            unsafe {
                nvic.set_priority(Interrupt::TIM3, 1);
                cortex_m::peripheral::NVIC::unmask(Interrupt::TIM3);
            }
        
        cortex_m::peripheral::NVIC::unpend(Interrupt::TIM3);

        // Set up the display

        let mut disp: TerminalMode<_> = SSD1306Builder::new().size(DisplaySize::Display128x32).connect_i2c(i2c).into();
        
        disp.init().unwrap();

        disp.clear().unwrap();

    });

    
    // this loop will not work as it cannot access disp or delay
    
    loop {

        let mut buffer = ArrayString::<[u8; 64]>::new();

        let input = free(|cs| COUNTER.borrow(cs).get());
    
        format(&mut buffer, input);
        
        disp.write_str(buffer.as_str());
                
        delay.delay_ms(200_u16);
    
    }
    
}

    loop {continue;}
    
}


// string formatting function, 64 characters to fill up the whole display

fn format(buf: &mut ArrayString<[u8; 64]>, value: u8) {
    fmt::write(buf, format_args!("value: {:04}                                                     ", value)).unwrap();
}