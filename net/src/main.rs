#![no_std]
#![no_main]

use core::sync::atomic::{AtomicUsize, Ordering};
use cortex_m::asm::delay;
use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _; // global logger
use nrf5340_net_pac as pac;
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

    if !p.UICR_NS.approtect.read().pall().is_unprotected() {
        info!("Setting UICR.APPROTECT=Unprotected");
        p.NVMC_NS.config.write(|w| w.wen().wen());
        while p.NVMC_NS.ready.read().bits() == 0 {}
        p.UICR_NS.approtect.write(|w| w.pall().unprotected());
        while p.NVMC_NS.ready.read().bits() == 0 {}
        p.NVMC_NS.config.write(|w| w.wen().ren());
    }

    p.CTRLAP_NS
        .approtect
        .disable
        .write(|w| unsafe { w.bits(0x50FA50FA) });

    p.P0_NS.pin_cnf[29].write(|w| w.dir().output());
    loop {
        cortex_m::asm::delay(10_000_000);
        p.P0_NS.outset.write(|w| w.pin29().set());
        cortex_m::asm::delay(10_000_000);
        p.P0_NS.outclr.write(|w| w.pin29().clear());
    }
}
