use std::convert::TryFrom;
use std::io::prelude::*;
use chrono::{DateTime, Local};

use crate::FloppyType;

#[derive(Clone, Debug)]
pub struct MapFile {
    start_time: Option<DateTime<Local>>,
    current_time: Option<DateTime<Local>>,
    current_pos: u64,
    status: Status,
    pass: u64,
    total_size: u64,
    blocks: Vec<Block>,
    floppy_type: FloppyType,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    CopyingNonTriedBlocks,
    TrimmingNonTrimmerBlocks,
    ScrapingNonScrapedBlocks,
    RetryingBadSectors,
    FillingSpecifiedBlocks,
    GeneratingApproximateMapfile,
    Finished,
}

#[derive(Debug)]
pub enum Error {
    NoStatusLine,
    UnknownFloppyType,
    InvalidLine(usize),
    Io(std::io::Error),
}

impl MapFile {
    pub fn load(reader: impl BufRead, floppy_type: Option<FloppyType>) -> Result<Self, Error> {
        fn parse_number(s: &str, line_index: usize) -> Result<u64, Error> {
            let (s, radix) = if s.starts_with("0x") {
                (&s[2..], 16)
            } else if s.starts_with("0") {
                (&s[1..], 8)
            } else {
                (s, 10)
            };

            u64::from_str_radix(s, radix).map_err(|_| Error::InvalidLine(line_index + 1))
        }

        fn try_parse_comment_line<T, E>(
            s: &str,
            prefix: &str,
            value: &mut Option<T>,
            parse: impl FnOnce(&str) -> Result<T, E>)
        {
            if value.is_none() && s.starts_with(prefix) {
                *value = parse(&s[prefix.len()..].trim()).ok();
            }
        }

        fn parse_datetime(s: &str) -> chrono::ParseResult<DateTime<Local>> {
            DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").map(|d| d.into())
        }

        const COMMENT_START_TIME: &str = "# Start time:";
        const COMMENT_CURRENT_TIME: &str = "# Current time:";

        let mut start_time = None;
        let mut current_time = None;
        let mut status_line: Option<(u64, Status, u64)> = None;
        let mut total_size = 0;
        let mut blocks = Vec::new();

        for (index, line) in reader.lines().enumerate() {
            let line = line.map_err(Error::Io)?;
            let line = line.trim();

            if line.starts_with("#") {
                try_parse_comment_line(line, COMMENT_START_TIME, &mut start_time, parse_datetime);
                try_parse_comment_line(line, COMMENT_CURRENT_TIME, &mut current_time, parse_datetime);
            } else {
                let parts = line.split_whitespace().collect::<Vec<_>>();
                if parts.len() != 3 {
                    return Err(Error::InvalidLine(index + 1));
                }

                if status_line.is_none() {
                    let current_pos = parse_number(parts[0], index)?;
                    let status = match parts[1] {
                        "?" => Status::CopyingNonTriedBlocks,
                        "*" => Status::TrimmingNonTrimmerBlocks,
                        "/" => Status::ScrapingNonScrapedBlocks,
                        "-" => Status::RetryingBadSectors,
                        "F" => Status::FillingSpecifiedBlocks,
                        "G" => Status::GeneratingApproximateMapfile,
                        "+" => Status::Finished,
                        _ => return Err(Error::InvalidLine(index + 1)),
                    };
                    let pass = parse_number(parts[2], index)?;

                    status_line = Some((current_pos, status, pass));
                } else {
                    let pos = parse_number(parts[0], index)?;
                    let size = parse_number(parts[1], index)?;

                    let status = match parts[2] {
                        "?" => BlockStatus::NonTried,
                        "*" => BlockStatus::NonTrimmed,
                        "/" => BlockStatus::NonScraped,
                        "-" => BlockStatus::BadSector,
                        "+" => BlockStatus::Finished,
                        _ => return Err(Error::InvalidLine(index + 1)),
                    };

                    blocks.push(Block {
                        pos,
                        size,
                        status,
                    });

                    total_size += size;
                }
            }
        }

        let floppy_type = floppy_type
            .or_else(|| FloppyType::find_by_total_size(total_size).cloned())
            .ok_or(Error::UnknownFloppyType)?;

        if let Some((current_pos, status, pass)) = status_line {
            Ok(MapFile {
                start_time,
                current_time,
                current_pos,
                status,
                pass,
                total_size,
                blocks,
                floppy_type,
            })
        } else {
            Err(Error::NoStatusLine)
        }
    }

    pub fn start_time(&self) -> Option<DateTime<Local>> {
        self.start_time
    }

    pub fn current_time(&self) -> Option<DateTime<Local>> {
        self.current_time
    }

    pub fn current_pos(&self) -> u64 {
        self.current_pos
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn pass(&self) -> u64 {
        self.pass
    }

    pub fn total_size(&self) -> u64 {
        self.total_size
    }

    pub fn floppy_type(&self) -> &FloppyType {
        &self.floppy_type
    }

    /*pub fn blocks(&self) -> std::slice::Iter<Block> {
        self.blocks.iter()
    }*/

    pub fn sectors(&self) -> Sectors {
        Sectors {
            map: self,
            block: 0,
            index: 0,
            side: 0,
            track: 0,
            sector: 0,
        }
    }
}

pub struct Sectors<'a> {
    map: &'a MapFile,
    block: usize,
    index: u64,
    side: u64,
    track: u64,
    sector: u64,
}

impl Iterator for Sectors<'_> {
    type Item = Sector;

    fn next(&mut self) -> Option<Self::Item> {
        let floppy = &self.map.floppy_type;

        while let Some(block) = self.map.blocks.get(self.block) {
            let pos = self.index * floppy.sector_size;
            let block_pos = pos - block.pos();

            if block_pos < block.size() {
                let r = Some(Sector {
                    pos,
                    index: self.index,
                    side: self.side,
                    track: self.track,
                    sector: self.sector + 1,
                    status: block.status,
                });

                self.sector += 1;
                if self.sector == floppy.sectors {
                    self.sector = 0;
                    self.track += 1;

                    if self.track == floppy.tracks {
                        self.track = 0;
                        self.side += 1;
                    }
                }

                self.index += 1;

                return r;
            }

            self.block += 1;
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let sector_size = self.map.floppy_type.sector_size;
        let total_sectors = (self.map.total_size + sector_size - 1) / sector_size;

        let hint = usize::try_from(total_sectors - self.index)
            .expect("this disk is absurdly big");

        (hint, Some(hint))
    }
}

impl ExactSizeIterator for Sectors<'_> { }

#[derive(Clone, Debug)]
struct Block {
    pos: u64,
    size: u64,
    status: BlockStatus,
}

impl Block {
    pub fn pos(&self) -> u64 {
        self.pos
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn status(&self) -> BlockStatus {
        self.status
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockStatus {
    NonTried,
    NonTrimmed,
    NonScraped,
    BadSector,
    Finished,
}

pub struct Sector {
    pub pos: u64,
    pub index: u64,
    pub side: u64,
    pub track: u64,
    pub sector: u64,
    pub status: BlockStatus,
}

