use crate::{take1, BlockHeader};
use nom::Parser as _;

#[derive(Debug)]
pub struct Block {
    pub header: BlockHeader,
    pub inner: Option<BlockImpl>,
}

#[derive(Debug)]
pub enum BlockImpl {
    RarVolumeHeader,
    RarPackedFile(RarPackedFile),
    RarOldRecovery(RarOldRecovery),
    RarNewSub,

    //srr
    SrrHeader,
    SrrStoredFile(SrrStoredFile),
    SrrRarFile(SrrRarFile),

    //new
    SrrOsoHash,
    SrrRarPadding,
}

#[derive(Debug)]
pub struct RarOldRecovery {
    pub packed_size: u32,
    pub rar_version: u8,
    pub recovery_sector: u16,
    pub data_sectors: u32,
}

impl RarOldRecovery {
    pub fn parse(input: &[u8]) -> nom::IResult<&[u8], Self> {
        let (rest, packed_size) = nom::number::le_u32().parse(input)?;
        let (rest, rar_version) = take1(rest)?;
        let (rest, recovery_sector) = nom::number::le_u16().parse(rest)?;
        let (rest, data_sectors) = nom::number::le_u32().parse(rest)?;
        Ok((
            rest,
            Self {
                packed_size,
                rar_version,
                recovery_sector,
                data_sectors,
            },
        ))
    }
}

#[derive(Debug)]
pub struct SrrStoredFile {
    pub file_name: String,
}

impl SrrStoredFile {
    pub fn new(input: &[u8]) -> nom::IResult<&[u8], Self> {
        let (rest, name_length) = nom::number::le_u16().parse(input)?;
        let (rest, file_name) = nom::bytes::complete::take(name_length)
            .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
            .parse(rest)?;
        Ok((rest, Self { file_name }))
    }
}

#[derive(Debug)]
pub struct SrrRarFile {
    pub file_name: String,
}

impl SrrRarFile {
    pub fn new(input: &[u8]) -> nom::IResult<&[u8], Self> {
        let (rest, name_length) = nom::number::le_u16().parse(input)?;
        let (rest, file_name) = nom::bytes::complete::take(name_length)
            .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
            .parse(rest)?;
        Ok((rest, Self { file_name }))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DateTime {
    pub year: u16,
    pub month: u16,
    pub day: u16,
    pub hour: u16,
    pub minute: u16,
    pub second: u16,
}

impl DateTime {
    fn parse(stamp: u32) -> Self {
        let second = (stamp & 0x1F) * 2;
        let stamp = stamp >> 5;
        let minute = stamp & 0x3F;
        let stamp = stamp >> 6;
        let hour = stamp & 0x1F;
        let stamp = stamp >> 5;
        let day = stamp & 0x1F;
        let stamp = stamp >> 5;
        let month = stamp & 0x0F;
        let year = (stamp >> 4) & 0x7F + 1980;
        Self {
            year: year as u16,
            month: month as u16,
            day: day as u16,
            hour: hour as u16,
            minute: minute as u16,
            second: second as u16,
        }
    }
}

#[derive(Debug)]
pub enum HostOS {
    MsDOS = 0,
    OS2 = 1,
    Windows = 2,
    Unix = 3,
    MacOS = 4,
    BeOS = 5,
}

impl TryFrom<u8> for HostOS {
    type Error = u8;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let v = match value {
            0 => Self::MsDOS,
            1 => Self::OS2,
            2 => Self::Windows,
            3 => Self::Unix,
            4 => Self::MacOS,
            5 => Self::BeOS,
            _ => {
                return Err(value);
            }
        };
        Ok(v)
    }
}

#[derive(Debug)]
pub struct RarPackedFile {
    pub unpacked_size: u32,
    pub os: HostOS,
    pub file_crc: u32,
    pub datetime: DateTime,
    pub unpack_version: u8,
    pub compression_method: u8,
    pub file_attributes: u32,
    pub file_name: String,
    pub salt: u64,
}

impl RarPackedFile {
    pub fn parse<'a>(input: &'a [u8], header: &BlockHeader) -> nom::IResult<&'a [u8], Self> {
        fn parse_xtime(
            flag: u16,
            data: &[u8],
            dostime: Option<DateTime>,
        ) -> nom::IResult<&[u8], Option<DateTime>> {
            if flag & 8 != 0 {
                let (mut data, mut dostime) = if let Some(dostime) = dostime {
                    (data, dostime)
                } else {
                    nom::number::le_u32().map(DateTime::parse).parse(data)?
                };

                let mut rem: u32 = 0;
                let cnt = flag & 3;
                for _ in 0..cnt {
                    let (rest, b) = take1(data)?;
                    rem = ((b as u32) << 16) | (rem >> 8);
                    data = rest;
                }

                let mut sec = (rem / 10000000) as u16; // 100ns
                if flag & 4 != 0 {
                    sec += 1;
                }
                dostime.second += sec;

                Ok((data, Some(dostime)))
            } else {
                Ok((data, None))
            }
        }

        let (rest, unpacked_size) = nom::number::le_u32().parse(input)?;
        let (rest, os) = take1(rest).and_then(|(rest, v)| {
            let os = HostOS::try_from(v).map_err(|_err| {
                nom::Err::Error(nom::error::make_error(rest, nom::error::ErrorKind::Tag))
            })?;
            Ok((rest, os))
        })?;
        let (rest, file_crc) = nom::number::le_u32().parse(rest)?;
        let (rest, datetime) = nom::number::le_u32().map(DateTime::parse).parse(rest)?;
        let (rest, unpack_version) = take1(rest)?;
        let (rest, compression_method) = take1(rest)?;
        let (rest, name_length) = nom::number::le_u16().parse(rest)?;
        let (rest, file_attributes) = nom::number::le_u32().parse(rest)?;

        let (rest, _) = if (header.flags & 0x100) != 0 {
            let (rest, high_packed_size) = nom::number::le_u32().parse(rest)?;
            let (rest, high_unpacked_size) = nom::number::le_u32().parse(rest)?;

            let high_packed_size = high_packed_size as u64 * 0x100000000;
            let high_unpacked_size = high_unpacked_size as u64 * 0x100000000;
            (rest, (high_packed_size, high_unpacked_size))
        } else {
            (rest, (0, 0))
        };

        // if self.flags & RarPackedFileBlock.UTF8_FILE_NAME: // 0x0200
        // 	null = self.file_name.find(ZERO) # index zero byte
        // 	self.orig_filename = self.file_name[:null]
        // 	u = UnicodeFilename(self.orig_filename, self.file_name[null + 1:])
        // 	self.unicode_filename = u.decode()
        // else:
        // 	self.orig_filename = self.file_name
        // 	self.unicode_filename = self.file_name.decode(DEFAULT_CHARSET, "replace")

        let (rest, file_name) = nom::bytes::take(name_length)
            .map(|data| {
                let untrimmed = String::from_utf8_lossy(data);
                match untrimmed.split_once('\0') {
                    Some((file_name, _term)) => file_name.to_string(),
                    None => untrimmed.to_string(),
                }
            })
            .parse(rest)?;

        let (rest, salt) = if header.flags & 0x400 != 0 {
            nom::number::le_u64().parse(rest)?
        } else {
            (rest, 0)
        };

        let (rest, _time) = if header.flags & 0x1000 != 0 {
            let (rest, flags) = if rest.len() >= 2 {
                // println!("{:0>2X?}", &rest[..2]);
                nom::number::le_u16().parse(rest)?
            } else {
                (rest, 0)
            };

            // println!("{:0>2X?}", &rest[..3]);
            // println!("{}", rest.len());
            let (rest, _modification_time) = parse_xtime(flags >> 3 * 4, rest, Some(datetime))?;
            // println!("{}", rest.len());
            let (rest, _creation_time) = parse_xtime(flags >> 2 * 4, rest, None)?;
            // println!("{}", rest.len());
            let (rest, _last_access_time) = parse_xtime(flags >> 1 * 4, rest, None)?;
            // println!("{}", rest.len());
            let (rest, _archival_time) = parse_xtime(flags >> 0 * 4, rest, None)?;
            // println!("{}", rest.len());
            (rest, 0)
        } else {
            (rest, 0)
        };

        Ok((
            rest,
            Self {
                unpacked_size,
                os,
                file_crc,
                datetime,
                unpack_version,
                compression_method,
                file_attributes,
                file_name,
                salt,
            },
        ))
    }
}
