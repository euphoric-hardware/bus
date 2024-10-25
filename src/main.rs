use std::collections::BTreeMap;

#[derive(Debug)]
enum Error {
    InvalidAccess,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Default)]
struct MemoryRange {
    base_address: u64,
    size: u64,
}

impl PartialEq for MemoryRange {
    fn eq(&self, other: &Self) -> bool {
        self.base_address.eq(&other.base_address)
    }
}

impl Eq for MemoryRange {}

impl PartialOrd for MemoryRange {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.base_address.partial_cmp(&other.base_address)
    }
}

impl Ord for MemoryRange {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.base_address.cmp(&other.base_address)
    }
}

impl MemoryRange {
    fn contains(&self, addr: u64) -> bool {
        addr >= self.base_address && addr < self.base_address + self.size
    }
}

trait Device {
    fn read(&self, ptr: u64, buf: &mut [u8], len: u64) -> Result<()>;
    fn write(&mut self, ptr: u64, buf: &[u8], len: u64) -> Result<()>;
}

struct Bus<'b> {
    devices: BTreeMap<MemoryRange, Box<dyn Device + 'b>>,
}

impl<'b> Bus<'b> {
    fn get_device(&self, addr: u64) -> Result<(&MemoryRange, &dyn Device)> {
        self.devices
            .range(
                ..=MemoryRange {
                    base_address: addr,
                    ..Default::default()
                },
            )
            .rev()
            .find(|(range, _)| range.contains(addr)) // should be first
            .map(|(r, device)| (r, &**device))
            .ok_or(Error::InvalidAccess)
    }

    fn get_device_mut(&mut self, addr: u64) -> Result<(&MemoryRange, &mut (dyn Device + 'b))> {
        self.devices
            .range_mut(
                ..=MemoryRange {
                    base_address: addr,
                    ..Default::default()
                },
            )
            .rev()
            .find(|(range, _)| range.contains(addr)) // should be first
            .map(|(r, device)| (r, &mut **device))
            .ok_or(Error::InvalidAccess)
    }

    fn register(&mut self, device: Box<dyn Device>, base_address: u64, size: u64) {
        self.devices
            .insert(MemoryRange { base_address, size }, device);
    }
}

impl Device for Bus<'_> {
    fn read(&self, ptr: u64, buf: &mut [u8], len: u64) -> Result<()> {
        let device = self.get_device(ptr)?;
        device.1.read(ptr - device.0.base_address, buf, len)
    }

    fn write(&mut self, ptr: u64, buf: &[u8], len: u64) -> Result<()> {
        let device = self.get_device_mut(ptr)?;
        device.1.write(ptr - device.0.base_address, buf, len)
    }
}

struct Ram {
    data: [u8; 1024],
}

impl Ram {
    const BASE_ADDRESS: u64 = 0x10000;
    const SIZE: u64 = 0x10000;
}

impl Device for Ram {
    fn read(&self, ptr: u64, buf: &mut [u8], len: u64) -> Result<()> {
        buf.copy_from_slice(&self.data[ptr as usize..ptr as usize + len as usize]);
        Ok(())
    }

    fn write(&mut self, ptr: u64, buf: &[u8], len: u64) -> Result<()> {
        self.data[ptr as usize..ptr as usize + len as usize].copy_from_slice(&buf);
        Ok(())
    }
}

fn main() {
    let mut bus = Bus {
        devices: BTreeMap::new(),
    };
    let ram = Box::new(Ram { data: [0; 1024] });

    bus.register(ram, Ram::BASE_ADDRESS, Ram::SIZE);
    bus.write(Ram::BASE_ADDRESS + 1, &[1, 2, 3], 3).unwrap();

    let mut buf = [0; 3];
    bus.read(Ram::BASE_ADDRESS + 1, &mut buf, 3).unwrap();

    println!("{:?}", buf);
}
