#![no_std]
#![no_main]

use core::sync::atomic::{AtomicUsize, Ordering};
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

    p.SPU_S.periphid[66]
        .perm
        .write(|w| w.secattr().non_secure());
    p.SPU_S.gpioport[0].perm.write(|w| unsafe { w.bits(0) });

    p.P0_S.pin_cnf[29].write(|w| w.mcusel().network_mcu());

    // Boot network core
    p.RESET_S.network.forceoff.write(|w| w.forceoff().release());

    p.P0_NS.pin_cnf[28].write(|w| w.dir().output());
    loop {
        p.P0_NS.outclr.write(|w| w.pin28().clear());
        cortex_m::asm::delay(10_000_000);
        p.P0_NS.outset.write(|w| w.pin28().set());
        cortex_m::asm::delay(10_000_000);
    }
}
