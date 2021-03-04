#![no_std]
#![no_main]

use core::sync::atomic::{AtomicUsize, Ordering};
use cortex_m::asm::delay;
use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _; // global logger
use nrf5340_app_pac as pac;
use panic_probe as _;

defmt::timestamp! {"{=u64}", {
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        // NOTE(no-CAS) `timestamps` runs with interrupts disabled
        let n = COUNT.load(Ordering::Relaxed);
        COUNT.store(n + 1, Ordering::Relaxed);
        n as u64
    }
}

#[entry]
fn main() -> ! {
    info!("Hello asd!");

    let p = pac::Peripherals::take().unwrap();

    p.CACHE_S.enable.write(|w| w.enable().enabled());

    p.CLOCK_S.hfclkctrl.write(|w| w.hclk().div1());
    //p.CLOCK_S.hfclksrc.write(|w| w.src().hfxo());

    if !p.UICR_S.approtect.read().pall().is_unprotected() {
        info!("Setting UICR.APPROTECT=Unprotected");
        p.NVMC_S.config.write(|w| w.wen().wen());
        while p.NVMC_S.ready.read().bits() == 0 {}
        p.UICR_S.approtect.write(|w| w.pall().unprotected());
        while p.NVMC_S.ready.read().bits() == 0 {}
        p.NVMC_S.config.write(|w| w.wen().ren());
    }

    if !p.UICR_S.secureapprotect.read().pall().is_unprotected() {
        info!("Setting UICR.SECUREAPPROTECT=Unprotected");
        p.NVMC_S.config.write(|w| w.wen().wen());
        while p.NVMC_S.ready.read().bits() == 0 {}
        p.UICR_S.secureapprotect.write(|w| w.pall().unprotected());
        while p.NVMC_S.ready.read().bits() == 0 {}
        p.NVMC_S.config.write(|w| w.wen().ren());
    }

    p.CTRLAP_S
        .approtect
        .disable
        .write(|w| unsafe { w.bits(0x50FA50FA) });
    p.CTRLAP_S
        .secureapprotect
        .disable
        .write(|w| unsafe { w.bits(0x50FA50FA) });

    // Boot network core
    p.RESET_S.network.forceoff.write(|w| w.forceoff().release());

    p.P0_S.pin_cnf[28].write(|w| w.dir().output());
    p.P0_S.pin_cnf[29].write(|w| w.dir().output());
    p.P0_S.pin_cnf[30].write(|w| w.dir().output());
    p.P0_S.pin_cnf[31].write(|w| w.dir().output());

    loop {
        p.P0_S.out.write(|w| {
            w.pin28().low();
            w.pin29().high();
            w.pin30().high();
            w.pin31().high();
            w
        });

        cortex_m::asm::delay(10_000_000);

        p.P0_S.out.write(|w| {
            w.pin28().high();
            w.pin29().low();
            w.pin30().high();
            w.pin31().high();
            w
        });

        cortex_m::asm::delay(10_000_000);

        p.P0_S.out.write(|w| {
            w.pin28().high();
            w.pin29().high();
            w.pin30().low();
            w.pin31().high();
            w
        });

        cortex_m::asm::delay(10_000_000);

        p.P0_S.out.write(|w| {
            w.pin28().high();
            w.pin29().high();
            w.pin30().high();
            w.pin31().low();
            w
        });

        cortex_m::asm::delay(10_000_000);
    }
}
