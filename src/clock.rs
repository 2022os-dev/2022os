#[allow(unused)]
pub fn clock_init() {
    #[cfg(feature = "board_unleashed")]
    board_clock_init();
}

#[cfg(feature = "board_unleashed")]
fn board_clock_init() {
    // const hfclk: usize = 33330000; // 33.33 MHz
    // 使用corepll使coreclk为 1G Hz(1000_000_000)
    // pll configuration base = 1000_0000 + 0x4
    // offset bit:
    //          divr [0:5]
    //          divf [6:14]
    //          divq [15:17]
    let divr = 0;
    let divf = 59;
    let divq = 2;
    let div = (divq << 15) + (divf << 6) + divr;
    let corepll = <*mut i32>::from_bits(0x1000_0000 + 0x4);
    unsafe {
        let mut i = corepll.read_volatile();
        i &= !((1 << 18) - 1);
        i += div;
        corepll.write_volatile(i);
        let mut i = corepll.read_volatile();
        // 等待信号稳定
        while i > 0 {
            i = corepll.read_volatile();
        }
    }
    let pllsel = <*mut i32>::from_bits(0x1000_0000 + 0x24);
    unsafe {
        pllsel.write_volatile(0);
    }
}
