#[inline(always)]
pub unsafe fn get_msb_pos(x: u64) -> u64 {
    let mut ret = 0u64;
    std::arch::asm!(
        "lzcnt {ret:r}, {bits:r}",
        bits = in(reg) x,
        ret = out(reg) ret
    );
    64 - ret
}