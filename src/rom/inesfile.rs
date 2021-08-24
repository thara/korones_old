#[derive(BinRead, Debug)]
#[br(magic = b"NES\x1A")]
pub(super) struct INESFile {
    prg_rom_unit_size: u8,
    chr_rom_unit_size: u8,
    pub(super) flag6: Flag6,
    flag7: Flag7,
    flag8: u8,
    flag9: Flag9,
    flag10: Flag10,

    #[br(pad_before = 5, count = prg_rom_unit_size as u16 * 0x4000u16)]
    pub(super) prg_rom: Vec<u8>,
    #[br(count = chr_rom_unit_size as u16 * 0x2000u16)]
    pub(super) chr_rom: Vec<u8>,

    #[br(calc = (flag7.bits() & 0b11110000) + (flag6.bits() >> 4))]
    pub(super) mapper: u8,

    #[br(calc = if 0 < flag8 { flag8 as u16 * 0x2000u16 } else { 0x2000u16 })]
    prg_ram_size: u16,
}

bitflags! {
    #[derive(BinRead, Default)]
    pub(super) struct Flag6: u8 {
        const MIRRORING_VERTICAL = 1 << 0;
        const BATTERY_BACKED_PRG_RAM = 1 << 1;
        const TRAINER_BEFORE_RPM_ROM = 1 << 2;  //TODO how to affect to INESFile.prg_rom's attribute?
        const FULL_SCREEN_VRAM = 1 << 3;
    }
}

impl Flag6 {
    #[allow(dead_code)]
    fn mapper_lower_nybble(&self) -> u8 {
        self.bits & 0b11110000
    }
}

bitflags! {
    #[derive(BinRead, Default)]
    struct Flag7: u8 {
        const VS_UNISYSTEM = 1 << 0;
        const PLAY_CHOICE_10 = 1 << 1;
        const NES2_FORMAT = 0b1100;
    }
}

impl Flag7 {
    #[allow(dead_code)]
    fn mapper_upper_nybble(&self) -> u8 {
        self.bits & 0b11110000
    }
}

bitflags! {
    #[derive(BinRead, Default)]
    struct Flag9: u8 {
        const TV_SYSTEM_PAL = 1 << 0;
    }
}

bitflags! {
    #[derive(BinRead, Default)]
    struct Flag10: u8 {
        const TV_SYSTEM_PAL = 1 << 1;
        const TV_SYSTEM_DUAL = 0b11;
        const PRG_RAM = 1 << 4;
        const BUS_CONFLICTED = 1 << 5;
    }
}

// use binread::{io::*, BinResult, ReadOptions};
// fn read_to_end<R: Read + Seek>(reader: &mut R, _: &ReadOptions, _: ()) -> BinResult<Vec<u8>> {
//     let mut buf = Vec::new();
//     let _ = reader.read_to_end(&mut buf);
//     Ok(buf)
// }

#[cfg(test)]
mod tests {
    use super::*;
    use binread::{io::Cursor, BinRead};

    #[test]
    fn load_ines_file() {
        use std::fs::File;
        use std::io::Read;
        use std::path::Path;

        let nes_dir = env!("CARGO_MANIFEST_DIR");
        let path = Path::new(nes_dir).join("roms/nes-test-roms/other/nestest.nes");

        let mut f = File::open(path).unwrap();
        let mut buf = Vec::new();
        let _ = f.read_to_end(&mut buf);
        let mut cur = Cursor::new(buf);

        let ines = INESFile::read(&mut cur).unwrap();
        assert_eq!(ines.prg_rom_unit_size, 1);
        assert_eq!(ines.chr_rom_unit_size, 1);

        assert_eq!(ines.prg_rom.len(), 0x4000);
        assert_eq!(ines.chr_rom.len(), 0x2000);

        assert_eq!(ines.mapper, 0);
    }
}
