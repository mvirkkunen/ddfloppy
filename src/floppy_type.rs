/// Floppy type.
#[derive(Clone, Debug)]
pub struct FloppyType {
    /// Name
    pub name: &'static str,

    /// Sides (1 or 2)
    pub sides: u64,

    /// Tracks per side
    pub tracks: u64,

    /// Sectors per track
    pub sectors: u64,

    /// Sector size in bytes
    pub sector_size: u64,

    /// Total size in bytes
    pub total_size: u64,
}

impl FloppyType {
    const fn new(name: &'static str, sides: u64, tracks: u64, sectors: u64) -> FloppyType {
        const SECTOR_SIZE: u64 = 512; // For now anyways

        FloppyType {
            name,
            sides,
            tracks,
            sectors,
            sector_size: SECTOR_SIZE,
            total_size: sides * tracks * sectors * SECTOR_SIZE,
        }
    }

    pub fn types() -> std::slice::Iter<'static, FloppyType> {
        FLOPPY_TYPES.iter()
    }

    pub fn find_by_total_size(total_size: u64) -> Option<&'static FloppyType> {
        FLOPPY_TYPES.iter().find(|t| t.total_size == total_size)
    }
}

const FLOPPY_TYPES: &[FloppyType] = &[
    FloppyType::new("5¼\" 160K", 1, 40, 8),
    FloppyType::new("5¼\" 320K", 2, 40, 8),
    FloppyType::new("5¼\" 180K", 1, 40, 9),
    FloppyType::new("5¼\" 360K", 2, 40, 9),
    //FloppyType::new("5¼\" 720K", 2, 80, 9),
    FloppyType::new("5¼\" 1.2M", 2, 80, 15),

    FloppyType::new("3½\" 720K", 2, 80, 9),
    FloppyType::new("3½\" 1.44K", 2, 80, 18),
    FloppyType::new("3½\" 1.72K", 2, 80, 21),
    FloppyType::new("3½\" 2.88K", 2, 80, 36),
];
