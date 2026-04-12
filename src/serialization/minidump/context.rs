use std::io::{self, Write};

use byteorder::{LittleEndian, WriteBytesExt};
use minidump::format as md;

use crate::arch::Arch;
use crate::flags::Flags;
use crate::regs64::Regs64;

const CONTEXT_X86_SIZE: usize = 716;
const CONTEXT_AMD64_SIZE: usize = 1232;

pub(super) fn build_thread_context(
    arch: Arch,
    regs: &Regs64,
    flags: &Flags,
) -> io::Result<Vec<u8>> {
    match arch {
        Arch::X86 => build_x86_context(regs, flags),
        Arch::X86_64 => build_amd64_context(regs, flags),
        Arch::Aarch64 => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "aarch64 thread export is not implemented yet",
        )),
    }
}

fn build_x86_context(regs: &Regs64, flags: &Flags) -> io::Result<Vec<u8>> {
    let mut output = Vec::with_capacity(CONTEXT_X86_SIZE);
    output.write_u32::<LittleEndian>(md::ContextFlagsX86::CONTEXT_X86_ALL.bits())?;
    output.write_u32::<LittleEndian>(regs.dr0 as u32)?;
    output.write_u32::<LittleEndian>(regs.dr1 as u32)?;
    output.write_u32::<LittleEndian>(regs.dr2 as u32)?;
    output.write_u32::<LittleEndian>(regs.dr3 as u32)?;
    output.write_u32::<LittleEndian>(regs.dr6 as u32)?;
    output.write_u32::<LittleEndian>(regs.dr7 as u32)?;

    output.write_u32::<LittleEndian>(0x027f)?;
    output.write_u32::<LittleEndian>(0)?;
    output.write_u32::<LittleEndian>(0)?;
    output.write_u32::<LittleEndian>(0)?;
    output.write_u32::<LittleEndian>(0)?;
    output.write_u32::<LittleEndian>(0)?;
    output.write_u32::<LittleEndian>(0)?;
    output.extend_from_slice(&[0; 80]);
    output.write_u32::<LittleEndian>(0)?;

    output.write_u32::<LittleEndian>(regs.gs as u32)?;
    output.write_u32::<LittleEndian>(if regs.fs == 0 { 0x3b } else { regs.fs as u32 })?;
    output.write_u32::<LittleEndian>(0x23)?;
    output.write_u32::<LittleEndian>(0x23)?;
    output.write_u32::<LittleEndian>(regs.get_edi() as u32)?;
    output.write_u32::<LittleEndian>(regs.get_esi() as u32)?;
    output.write_u32::<LittleEndian>(regs.get_ebx() as u32)?;
    output.write_u32::<LittleEndian>(regs.get_edx() as u32)?;
    output.write_u32::<LittleEndian>(regs.get_ecx() as u32)?;
    output.write_u32::<LittleEndian>(regs.get_eax() as u32)?;
    output.write_u32::<LittleEndian>(regs.get_ebp() as u32)?;
    output.write_u32::<LittleEndian>(regs.get_eip() as u32)?;
    output.write_u32::<LittleEndian>(0x1b)?;
    output.write_u32::<LittleEndian>(flags.dump())?;
    output.write_u32::<LittleEndian>(regs.get_esp() as u32)?;
    output.write_u32::<LittleEndian>(0x23)?;
    output.extend_from_slice(&[0; 512]);

    debug_assert_eq!(output.len(), CONTEXT_X86_SIZE);
    Ok(output)
}

fn build_amd64_context(regs: &Regs64, flags: &Flags) -> io::Result<Vec<u8>> {
    let mut output = Vec::with_capacity(CONTEXT_AMD64_SIZE);

    for _ in 0..6 {
        output.write_u64::<LittleEndian>(0)?;
    }

    output.write_u32::<LittleEndian>(md::ContextFlagsAmd64::CONTEXT_AMD64_ALL.bits())?;
    output.write_u32::<LittleEndian>(0x1f80)?;
    output.write_u16::<LittleEndian>(0x33)?;
    output.write_u16::<LittleEndian>(0)?;
    output.write_u16::<LittleEndian>(0)?;
    output.write_u16::<LittleEndian>(regs.fs as u16)?;
    output.write_u16::<LittleEndian>(regs.gs as u16)?;
    output.write_u16::<LittleEndian>(0x2b)?;
    output.write_u32::<LittleEndian>(flags.dump())?;
    output.write_u64::<LittleEndian>(regs.dr0)?;
    output.write_u64::<LittleEndian>(regs.dr1)?;
    output.write_u64::<LittleEndian>(regs.dr2)?;
    output.write_u64::<LittleEndian>(regs.dr3)?;
    output.write_u64::<LittleEndian>(regs.dr6)?;
    output.write_u64::<LittleEndian>(regs.dr7)?;
    output.write_u64::<LittleEndian>(regs.rax)?;
    output.write_u64::<LittleEndian>(regs.rcx)?;
    output.write_u64::<LittleEndian>(regs.rdx)?;
    output.write_u64::<LittleEndian>(regs.rbx)?;
    output.write_u64::<LittleEndian>(regs.rsp)?;
    output.write_u64::<LittleEndian>(regs.rbp)?;
    output.write_u64::<LittleEndian>(regs.rsi)?;
    output.write_u64::<LittleEndian>(regs.rdi)?;
    output.write_u64::<LittleEndian>(regs.r8)?;
    output.write_u64::<LittleEndian>(regs.r9)?;
    output.write_u64::<LittleEndian>(regs.r10)?;
    output.write_u64::<LittleEndian>(regs.r11)?;
    output.write_u64::<LittleEndian>(regs.r12)?;
    output.write_u64::<LittleEndian>(regs.r13)?;
    output.write_u64::<LittleEndian>(regs.r14)?;
    output.write_u64::<LittleEndian>(regs.r15)?;
    output.write_u64::<LittleEndian>(regs.rip)?;

    write_xmm_save_area32(&mut output, regs)?;

    for _ in 0..26 {
        write_u128(&mut output, 0)?;
    }

    for _ in 0..6 {
        output.write_u64::<LittleEndian>(0)?;
    }

    debug_assert_eq!(output.len(), CONTEXT_AMD64_SIZE);
    Ok(output)
}

fn write_xmm_save_area32(output: &mut Vec<u8>, regs: &Regs64) -> io::Result<()> {
    output.write_u16::<LittleEndian>(0x027f)?;
    output.write_u16::<LittleEndian>(0)?;
    output.write_u8(0)?;
    output.write_u8(0)?;
    output.write_u16::<LittleEndian>(0)?;
    output.write_u32::<LittleEndian>(0)?;
    output.write_u16::<LittleEndian>(0)?;
    output.write_u16::<LittleEndian>(0)?;
    output.write_u32::<LittleEndian>(0)?;
    output.write_u16::<LittleEndian>(0)?;
    output.write_u16::<LittleEndian>(0)?;
    output.write_u32::<LittleEndian>(0x1f80)?;
    output.write_u32::<LittleEndian>(0xffff)?;

    for mm in [
        regs.mm0, regs.mm1, regs.mm2, regs.mm3, regs.mm4, regs.mm5, regs.mm6, regs.mm7,
    ] {
        write_u128(output, mm)?;
    }

    for xmm in [
        regs.xmm0, regs.xmm1, regs.xmm2, regs.xmm3, regs.xmm4, regs.xmm5, regs.xmm6, regs.xmm7,
        regs.xmm8, regs.xmm9, regs.xmm10, regs.xmm11, regs.xmm12, regs.xmm13, regs.xmm14,
        regs.xmm15,
    ] {
        write_u128(output, xmm)?;
    }

    output.extend_from_slice(&[0; 96]);
    Ok(())
}

fn write_u128(output: &mut Vec<u8>, value: u128) -> io::Result<()> {
    output.write_all(&value.to_le_bytes())
}
