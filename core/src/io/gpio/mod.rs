mod rtc;

pub use rtc::RTC;


pub trait GPIO {
    fn clock(&mut self);
    fn process_write(&mut self);
    fn read(&self, byte: u8) -> u8;
    fn write(&mut self, byte: u8, value: u8);

    fn set_data0(&mut self, value: bool);
    fn set_data1(&mut self, value: bool);
    fn set_data2(&mut self, value: bool);
    fn set_data3(&mut self, value: bool);

    fn data0(&self) -> bool;
    fn data1(&self) -> bool;
    fn data2(&self) -> bool;
    fn data3(&self) -> bool;

    fn is_used(&self) -> bool;
    fn write_mask(&self) -> u8;
    fn can_write(&self, bit: u8) -> bool;
    fn set_write_mask(&mut self, value: u8);
    fn write_only(&self) -> bool;
    fn set_write_only(&mut self, value: bool);
}

impl dyn GPIO {
    pub fn read_register<D>(device: &D, offset: u32) -> u8 where D: GPIO {
        device.read(offset as u8)
    }

    pub fn write_register<D>(device: &mut D, offset: u32, value: u8) where D: GPIO {
        device.write(offset as u8, value);
    }
}
