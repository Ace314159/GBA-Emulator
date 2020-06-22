pub extern crate num_traits as num;

use std::mem::size_of;
use num::{cast::FromPrimitive, NumCast, PrimInt, Unsigned};
use super::{PPU, IO, IORegister};

impl MemoryHandler for IO {
    fn read<T>(&self, addr: u32) -> T where T: MemoryValue {
        match MemoryRegion::get_region(addr) {
            MemoryRegion::BIOS => IO::read_mem(&self.bios, addr),
            MemoryRegion::EWRAM => IO::read_mem(&self.ewram, addr & IO::EWRAM_MASK),
            MemoryRegion::IWRAM => IO::read_mem(&self.iwram, addr & IO::IWRAM_MASK),
            MemoryRegion::IO => IO::read_from_bytes(self, &IO::read_io_register, addr),
            MemoryRegion::Palette => IO::read_from_bytes(&self.ppu, &PPU::read_palette_ram, addr),
            MemoryRegion::VRAM => IO::read_mem(&self.ppu.vram, PPU::parse_vram_addr(addr)),
            MemoryRegion::OAM => IO::read_mem(&self.ppu.oam, PPU::parse_oam_addr(addr)),
            MemoryRegion::ROM0L | MemoryRegion::ROM0H |
            MemoryRegion::ROM1L | MemoryRegion::ROM1H |
            MemoryRegion::ROM2L => self.read_rom(addr),
            MemoryRegion::ROM2H => if self.cart_backup.is_eeprom_access(addr, self.rom.len()) && size_of::<T>() == 2 {
                if self.dma.in_dma {
                    FromPrimitive::from_u16(self.cart_backup.read_eeprom(addr)).unwrap()
                } else { num::one() }
            } else { self.read_rom(addr) },
            MemoryRegion::SRAM => self.read_sram(addr),
            MemoryRegion::Unused => { warn!("Reading Unused Memory at {:08X}", addr); num::zero() }
        }
    }

    fn write<T>(&mut self, addr: u32, value: T) where T: MemoryValue {
        match MemoryRegion::get_region(addr) {
            MemoryRegion::BIOS => (),
            MemoryRegion::EWRAM => IO::write_mem(&mut self.ewram, addr & IO::EWRAM_MASK, value),
            MemoryRegion::IWRAM => IO::write_mem(&mut self.iwram, addr & IO::IWRAM_MASK, value),
            MemoryRegion::IO => IO::write_from_bytes(self, &IO::write_register, addr, value),
            MemoryRegion::Palette => self.write_palette_ram(addr, value),
            MemoryRegion::VRAM => self.write_vram(PPU::parse_vram_addr(addr), value),
            MemoryRegion::OAM => self.write_oam(PPU::parse_oam_addr(addr), value),
            MemoryRegion::ROM0L | MemoryRegion::ROM0H |
            MemoryRegion::ROM1L | MemoryRegion::ROM1H |
            MemoryRegion::ROM2L => self.write_rom(addr, value),
            MemoryRegion::ROM2H => if self.cart_backup.is_eeprom_access(addr, self.rom.len()) {
                if self.dma.in_dma { self.cart_backup.write_eeprom(addr, num::cast::<T, u16>(value).unwrap()) }
                else { warn!("EEPROM Write not in DMA!") }
            } else { self.write_rom(addr, value) },
            MemoryRegion::SRAM => self.write_sram(addr, value),
            MemoryRegion::Unused => warn!("Writng Unused Memory at {:08X} {:08X}", addr, num::cast::<T, u32>(value).unwrap()),
        }
    }
}

pub enum MemoryRegion {
    BIOS,
    EWRAM,
    IWRAM,
    IO,
    Palette,
    VRAM,
    OAM,
    ROM0L,
    ROM0H,
    ROM1L,
    ROM1H,
    ROM2L,
    ROM2H,
    SRAM,
    Unused,
}

impl MemoryRegion {
    pub fn get_region(addr: u32) -> MemoryRegion {
        match addr >> 24 {
            0x00 if addr < 0x00004000 => MemoryRegion::BIOS, // Not Mirrored
            0x02 => MemoryRegion::EWRAM,
            0x03 => MemoryRegion::IWRAM,
            0x04 => MemoryRegion::IO,
            0x05 => MemoryRegion::Palette,
            0x06 => MemoryRegion::VRAM,
            0x07 => MemoryRegion::OAM,
            0x08 => MemoryRegion::ROM0L,
            0x09 => MemoryRegion::ROM0H,
            0x0A => MemoryRegion::ROM1L,
            0x0B => MemoryRegion::ROM1H,
            0x0C => MemoryRegion::ROM2L,
            0x0D => MemoryRegion::ROM2H,
            0x0E => MemoryRegion::SRAM,
            _ => MemoryRegion::Unused,
        }
    }
}

impl IO {
    fn read_mem<T>(mem: &Vec<u8>, addr: u32) -> T where T: MemoryValue {
        unsafe {
            *(&mem[addr as usize] as *const u8 as *const T)
        }
    }

    fn write_mem<T>(mem: &mut Vec<u8>, addr: u32, value: T) where T: MemoryValue {
        unsafe {
            *(&mut mem[addr as usize] as *mut u8 as *mut T) = value;
        }
    }

    fn read_from_bytes<T, F, D>(device: &D, read_fn: &F, addr: u32) -> T
        where T: MemoryValue, F: Fn(&D, u32) -> u8 {
        let mut value: T = num::zero();
        for i in 0..(size_of::<T>() as u32) {
            value = num::cast::<u8, T>(read_fn(device, addr + i)).unwrap() << (8 * i as usize) | value;
        }
        value
    }

    fn write_from_bytes<T, F, D>(device: &mut D, write_fn: &F, addr: u32, value: T)
        where T: MemoryValue, F: Fn(&mut D, u32, u8) {
        let mask = FromPrimitive::from_u8(0xFF).unwrap();
        for i in 0..size_of::<T>() {
            write_fn(device, addr + i as u32, num::cast::<T, u8>(value >> 8 * i & mask).unwrap());
        }
    }

    fn read_io_register(&self, addr: u32) -> u8 {
        match addr {
            0x04000000 ..= 0x0400005F => self.ppu.read_register(addr),
            0x04000060 ..= 0x040000AF => self.apu.read_register(addr),
            0x040000B0 ..= 0x040000BB => self.dma.channels[0].read(addr as u8 - 0xB0),
            0x040000BC ..= 0x040000C7 => self.dma.channels[1].read(addr as u8 - 0xBC),
            0x040000C8 ..= 0x040000D3 => self.dma.channels[2].read(addr as u8 - 0xC8),
            0x040000D4 ..= 0x040000DF => self.dma.channels[3].read(addr as u8 - 0xD4),
            0x04000100 ..= 0x04000103 => self.timers.timers[0].read(addr as u8 % 4),
            0x04000104 ..= 0x04000107 => self.timers.timers[1].read(addr as u8 % 4),
            0x04000108 ..= 0x0400010B => self.timers.timers[2].read(addr as u8 % 4),
            0x0400010C ..= 0x0400010F => self.timers.timers[3].read(addr as u8 % 4),
            0x04000130 => self.keypad.keyinput.read(0),
            0x04000131 => self.keypad.keyinput.read(1),
            0x04000132 => self.keypad.keycnt.read(0),
            0x04000133 => self.keypad.keycnt.read(1),
            0x04000200 => self.interrupt_controller.enable.read(0),
            0x04000201 => self.interrupt_controller.enable.read(1),
            0x04000202 => self.interrupt_controller.request.read(0),
            0x04000203 => self.interrupt_controller.request.read(1),
            0x04000204 => self.waitcnt.read(0),
            0x04000205 => self.waitcnt.read(1),
            0x04000206 ..= 0x04000207 => 0, // Unused IO Register
            0x04000208 => self.interrupt_controller.master_enable.read(0),
            0x04000209 => self.interrupt_controller.master_enable.read(1),
            0x0400020A ..= 0x040002FF => 0, // Unused IO Register
            0x04000300 => self.haltcnt as u8,
            0x04000301 => (self.haltcnt >> 8) as u8,
            0x04FFF780 ..= 0x04FFF781 => self.mgba_test_suite.read_register(addr),
            _ => { warn!("Reading Unimplemented IO Register at {:08X}", addr); 0 }
        }
    }

    fn write_register(&mut self, addr: u32, value: u8) {
        match addr {
            0x04000000 ..= 0x0400005F => self.ppu.write_register(addr, value),
            0x04000060 ..= 0x040000AF => self.apu.write_register(addr, value),
            0x040000B0 ..= 0x040000BB => self.dma.channels[0].write(addr as u8 - 0xB0, value),
            0x040000BC ..= 0x040000C7 => self.dma.channels[1].write(addr as u8 - 0xBC, value),
            0x040000C8 ..= 0x040000D3 => self.dma.channels[2].write(addr as u8 - 0xC8, value),
            0x040000D4 ..= 0x040000DF => self.dma.channels[3].write(addr as u8 - 0xD4, value),
            0x04000100 ..= 0x04000103 => self.timers.timers[0].write(addr as u8 % 4, value),
            0x04000104 ..= 0x04000107 => self.timers.timers[1].write(addr as u8 % 4, value),
            0x04000108 ..= 0x0400010B => self.timers.timers[2].write(addr as u8 % 4, value),
            0x0400010C ..= 0x0400010F => self.timers.timers[3].write(addr as u8 % 4, value),
            0x04000130 => self.keypad.keyinput.write(0, value),
            0x04000131 => self.keypad.keyinput.write(1, value),
            0x04000132 => self.keypad.keycnt.write(0, value),
            0x04000133 => self.keypad.keycnt.write(1, value),
            0x04000200 => self.interrupt_controller.enable.write(0, value),
            0x04000201 => self.interrupt_controller.enable.write(1, value),
            0x04000202 => self.interrupt_controller.request.write(0, value),
            0x04000203 => self.interrupt_controller.request.write(1, value),
            0x04000204 => self.waitcnt.write(0, value),
            0x04000205 => self.waitcnt.write(1, value),
            0x04000206 ..= 0x04000207 => (), // Unused IO Register
            0x04000208 => self.interrupt_controller.master_enable.write(0, value),
            0x04000209 => self.interrupt_controller.master_enable.write(1, value),
            0x0400020A ..= 0x040002FF => (), // Unused IO Register
            0x04000300 => self.haltcnt = (self.haltcnt & !0x00FF) | value as u16,
            0x04000301 => self.haltcnt = (self.haltcnt & !0xFF00) | (value as u16) << 8,
            0x04FFF600 ..= 0x04FFF701 => self.mgba_test_suite.write_register(addr, value),
            0x04FFF780 ..= 0x04FFF781 => self.mgba_test_suite.write_enable(addr, value),
            _ => warn!("Writng Unimplemented IO Register at {:08X} = {:08X}", addr, 0),
        }
    }

    fn read_rom<T>(&self, addr: u32) -> T where T: MemoryValue {
        let addr = addr - 0x08000000;
        if (addr as usize) < self.rom.len() { IO::read_mem(&self.rom, addr) }
        else { warn!("Returning Invalid ROM Read at 0x{:08X}", addr + 0x08000000); num::zero() }
    }

    fn read_sram<T>(&self, addr: u32) -> T where T: MemoryValue {
        if self.cart_backup.is_eeprom() { return num::zero() }
        let byte = FromPrimitive::from_u8(self.read_cart_backup(addr - 0x0E000000)).unwrap();
        match size_of::<T>() {
            1 => byte,
            2 => byte * FromPrimitive::from_u16(0x0101).unwrap(),
            4 => byte * FromPrimitive::from_u32(0x01010101).unwrap(),
            _ => unreachable!(),
        }
    }

    fn write_palette_ram<T>(&mut self, addr: u32, value: T) where T: MemoryValue {
        if size_of::<T>() == 1 {
            let value = num::cast::<T, u8>(value).unwrap();
            self.ppu.write_palette_ram(addr & !0x1, value);
            self.ppu.write_palette_ram(addr | 0x1, value);
        } else {
            IO::write_from_bytes(&mut self.ppu, &PPU::write_palette_ram, addr, value)
        }
    }

    fn write_vram<T>(&mut self, addr: u32, value: T) where T: MemoryValue {
        if size_of::<T>() == 1 {
            let addr = (addr & !0x1) as usize;
            let value = num::cast::<T, u8>(value).unwrap();
            self.ppu.vram[addr] = value;
            self.ppu.vram[addr + 1] = value;
        } else {
            IO::write_mem(&mut self.ppu.vram, addr, value);
        }
    }

    fn write_oam<T>(&mut self, addr: u32, value: T) where T: MemoryValue {
        if size_of::<T>() == 1 { return }
        IO::write_mem(&mut self.ppu.oam, addr, value);
    }

    fn write_rom<T>(&mut self, _addr: u32, _value: T) where T: MemoryValue {}

    fn write_sram<T>(&mut self, addr: u32, value: T) where T: MemoryValue {
        if self.cart_backup.is_eeprom() { return }
        let mask = FromPrimitive::from_u8(0xFF).unwrap();
        self.write_cart_backup(addr - 0x0E000000, num::cast::<T, u8>(value.rotate_right(addr * 8) & mask).unwrap());
    }

    fn read_cart_backup(&self, addr: u32) -> u8 { self.cart_backup.read(addr) }
    fn write_cart_backup(&mut self, addr: u32, value: u8) { self.cart_backup.write(addr, value) }
}

pub trait MemoryValue: Unsigned + PrimInt + NumCast + FromPrimitive {}

impl MemoryValue for u8 {}
impl MemoryValue for u16 {}
impl MemoryValue for u32 {}

pub trait MemoryHandler {
    fn read<T>(&self, addr: u32) -> T where T: MemoryValue;
    fn write<T>(&mut self, addr: u32, value: T) where T: MemoryValue;
}
