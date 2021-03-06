use riscv::register::sstatus;
use core::mem::size_of;

#[repr(C)]
pub struct TrapFrame {
    pub general_reg: [usize; 32],

    // Privileged registers
    pub sstatus: usize,
    pub sepc: usize,
    pub satp: usize,

    pub kernel_satp: usize,
    pub kernel_sp: usize,
    pub trap_handler: usize,

    pub ra_backpu: usize,
}

impl core::ops::Index<&str> for TrapFrame {
    type Output = usize;
    fn index(&self, index: &str) -> &Self::Output {
        match index {
            "zero" => &self.general_reg[0],
            "ra" => &self.general_reg[1],
            "sp" => &self.general_reg[2],
            "gp" => &self.general_reg[3],
            "tp" => &self.general_reg[4],
            "t0" => &self.general_reg[5],
            "t1" => &self.general_reg[6],
            "t2" => &self.general_reg[7],
            "fp" => &self.general_reg[8],
            "s0" => &self.general_reg[8],
            "s1" => &self.general_reg[9],
            "a0" => &self.general_reg[10],
            "a1" => &self.general_reg[11],
            "a2" => &self.general_reg[12],
            "a3" => &self.general_reg[13],
            "a4" => &self.general_reg[14],
            "a5" => &self.general_reg[15],
            "a6" => &self.general_reg[16],
            "a7" => &self.general_reg[17],
            "s2" => &self.general_reg[18],
            "s3" => &self.general_reg[19],
            "s4" => &self.general_reg[20],
            "s5" => &self.general_reg[21],
            "s6" => &self.general_reg[22],
            "s7" => &self.general_reg[23],
            "s8" => &self.general_reg[24],
            "s9" => &self.general_reg[25],
            "s10" => &self.general_reg[26],
            "s11" => &self.general_reg[27],
            "t3" => &self.general_reg[28],
            "t4" => &self.general_reg[29],
            "t5" => &self.general_reg[30],
            "t6" => &self.general_reg[31],
            "sepc" => &self.sepc,
            "sstatus" => &self.sstatus,
            _ => {
                panic!("unspported trapframe index {}", index)
            }
        }
    }
}
impl core::ops::IndexMut<&str> for TrapFrame {
    fn index_mut(&mut self, index: &str) -> &mut Self::Output {
        match index {
            "zero" => &mut self.general_reg[0],
            "ra" => &mut self.general_reg[1],
            "sp" => &mut self.general_reg[2],
            "gp" => &mut self.general_reg[3],
            "tp" => &mut self.general_reg[4],
            "t0" => &mut self.general_reg[5],
            "t1" => &mut self.general_reg[6],
            "t2" => &mut self.general_reg[7],
            "fp" => &mut self.general_reg[8],
            "s0" => &mut self.general_reg[8],
            "s1" => &mut self.general_reg[9],
            "a0" => &mut self.general_reg[10],
            "a1" => &mut self.general_reg[11],
            "a2" => &mut self.general_reg[12],
            "a3" => &mut self.general_reg[13],
            "a4" => &mut self.general_reg[14],
            "a5" => &mut self.general_reg[15],
            "a6" => &mut self.general_reg[16],
            "a7" => &mut self.general_reg[17],
            "s2" => &mut self.general_reg[18],
            "s3" => &mut self.general_reg[19],
            "s4" => &mut self.general_reg[20],
            "s5" => &mut self.general_reg[21],
            "s6" => &mut self.general_reg[22],
            "s7" => &mut self.general_reg[23],
            "s8" => &mut self.general_reg[24],
            "s9" => &mut self.general_reg[25],
            "s10" => &mut self.general_reg[26],
            "s11" => &mut self.general_reg[27],
            "t3" => &mut self.general_reg[28],
            "t4" => &mut self.general_reg[29],
            "t5" => &mut self.general_reg[30],
            "t6" => &mut self.general_reg[31],
            "sepc" => &mut self.sepc,
            "sstatus" => &mut self.sstatus,
            _ => {
                panic!("unspported trapframe index {}", index)
            }
        }
    }
}

impl TrapFrame {
    pub fn init(&mut self, sp: usize, sepc: usize) {
        let mut sstatus_reg = sstatus::read();
        sstatus_reg.set_spp(sstatus::SPP::User);
        sstatus_reg.set_spie(true);
        self["sstatus"] = sstatus_reg.bits();
        self["sepc"] = sepc;
        self.trap_handler = crate::trap::trap_handler as usize;
        // ??????argv envp
        self["sp"] = sp - 2 * size_of::<usize>();
        // argc = 0
        self["a0"] = 0;
        // argv
        self["a1"] = sp - size_of::<usize>();
        // envp
        self["a2"] = self["a1"] - size_of::<usize>();
    }
}
