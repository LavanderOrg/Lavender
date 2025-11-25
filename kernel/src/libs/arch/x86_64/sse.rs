use crate::{info, libs::arch::x86_64::{CPU_CONTEXT, cpu::BasicFeaturesFlags, registers::*}, warning};

pub fn init() -> Result<(), ()> {
    if !unsafe {
        CPU_CONTEXT.info.as_ref().ok_or(())?
            .basic_features.as_ref().ok_or(())?
                .flags.contains(BasicFeaturesFlags::SSE) } {
        // TODO: liballoc is currently built using SSE instructions, so we cannot fallback to emulation yet.
        warning!("SSE not supported on this CPU, defaulting to emulation mode for floating point operations.");
        return Err(());
    }

    // Clear CR0.EM (Emulate Math Coprocessor) and set CR0.MP (Monitor Coprocessor)
    write_cr0(cr0() & !(1 << 2) | (1 << 1));
    // Set CR4.OSFXSR (Operating System Support for FXSAVE and FXRSTOR instructions) and CR4.OSXMMEEXCPT (Operating System Support for Unmasked SIMD Floating-Point Exceptions)
    write_cr4(cr4() | (1 << 9) | (1 << 10));
    info!("SSE support enabled.");
    Ok(())
}
