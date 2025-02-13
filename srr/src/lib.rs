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
    RarOldRecovery,
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
pub struct SrrStoredFile {
    pub file_name: String,
}

impl SrrStoredFile {
    pub fn new(input: &[u8]) -> Self {
        let name_length = u16::from_le_bytes(input[0..2].try_into().unwrap()) as usize;
        let file_name = String::from_utf8_lossy(&input[2..][..name_length]).into_owned();
        Self { file_name }
    }
}

#[derive(Debug)]
pub struct SrrRarFile {
    pub file_name: String,
}

impl SrrRarFile {
    pub fn new(input: &[u8]) -> Self {
        let name_length = u16::from_le_bytes(input[0..2].try_into().unwrap()) as usize;
        let file_name = String::from_utf8_lossy(&input[2..][..name_length]).into_owned();
        Self { file_name }
    }
}

#[derive(Debug)]
pub struct RarPackedFile {
    pub unpacked_size: u32,
    pub os: u8,
    pub file_crc: u32,
    pub datetime: u32,
    pub unpack_version: u8,
    pub compression_method: u8,
    pub file_attributes: u32,
    pub file_name: String,
}

impl RarPackedFile {
    pub fn parse(input: &[u8]) -> nom::IResult<&[u8], Self> {
        let (rest, unpacked_size) = nom::number::le_u32().parse(input)?;
        let (rest, os) = take1(rest)?;
        let (rest, file_crc) = nom::number::le_u32().parse(rest)?;
        let (rest, datetime) = nom::number::le_u32().parse(rest)?;
        let (rest, unpack_version) = take1(rest)?;
        let (rest, compression_method) = take1(rest)?;
        let (rest, name_length) = nom::number::le_u16().parse(rest)?;
        let (rest, file_attributes) = nom::number::le_u32().parse(rest)?;

        // if (header.flags & 0x100) != 0 {
        //     // let packed_size =
        //     //     u32::from_le_bytes(input[21..25].try_into().unwrap()) * 0x100000000;
        //     // let unpacked_size =
        //     //     u32::from_le_bytes(input[25..29].try_into().unwrap()) * 0x100000000;
        //     unimplemented!()
        // }

        let (rest, file_name) = nom::bytes::take(name_length)
            .map(|data| {
                let untrimmed = String::from_utf8_lossy(data);
                match untrimmed.split_once('\0') {
                    Some((file_name, _term)) => file_name.to_string(),
                    None => untrimmed.to_string(),
                }
            })
            .parse(rest)?;
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
            },
        ))
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum BlockType {
    RarVolumeHeader = 0x73,
    RarPackedFile = 0x74,
    RarOldRecovery = 0x78,
    RarNewSub = 0x7A,

    //not intresting in web (only for reconstruction)
    RarMin = 0x72, //"RAR Marker"
    RarMax = 0x7B, //"Archive end"
    OldComment = 0x75,
    OldAuthenticity1 = 0x76,
    OldSubblock = 0x77,
    OldAuthenticity2 = 0x79,

    //srr
    SrrHeader = 0x69,
    SrrStoredFile = 0x6A,
    SrrRarFile = 0x71,

    //new
    SrrOsoHash = 0x6B,
    SrrRarPadding = 0x6C,
}

impl TryFrom<u8> for BlockType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let ty = match value {
            0x73 => Self::RarVolumeHeader,
            0x74 => Self::RarPackedFile,
            0x78 => Self::RarOldRecovery,
            0x7A => Self::RarNewSub,
            0x72 => Self::RarMin,
            0x7B => Self::RarMax,
            0x75 => Self::OldComment,
            0x76 => Self::OldAuthenticity1,
            0x77 => Self::OldSubblock,
            0x79 => Self::OldAuthenticity2,
            0x69 => Self::SrrHeader,
            0x6A => Self::SrrStoredFile,
            0x71 => Self::SrrRarFile,
            0x6B => Self::SrrOsoHash,
            0x6C => Self::SrrRarPadding,
            _ => return Err(value),
        };
        Ok(ty)
    }
}

#[derive(Debug)]
pub struct BlockHeader {
    pub crc: u16,
    pub ty: BlockType,
    pub flags: u16,
    pub size: u16,
    pub add_size: u32,
}

fn take1(i: &[u8]) -> nom::IResult<&[u8], u8> {
    match i.split_first() {
        Some((v, i)) => Ok((i, *v)),
        None => Err(nom::Err::Incomplete(nom::Needed::new(1))),
    }
}

impl BlockHeader {
    pub fn full_size(&self) -> usize {
        self.size as usize + self.add_size as usize
    }

    pub fn parse(input: &[u8]) -> nom::IResult<&[u8], Self> {
        let (rest, crc) = nom::number::le_u16().parse(input)?;
        let (rest, ty) = take1(rest).and_then(|(rest, b)| match BlockType::try_from(b) {
            Ok(ty) => Ok((rest, ty)),
            Err(_err) => Err(nom::Err::Error(nom::error_position!(
                rest,
                nom::error::ErrorKind::Tag
            ))),
        })?;
        let (rest, flags) = nom::number::le_u16().parse(rest)?;
        let (rest, size) = nom::number::le_u16().parse(rest)?;

        let (rest, add_size) = if (flags & 0x8000) > 0
            || matches!(ty, BlockType::RarPackedFile | BlockType::RarNewSub)
        {
            nom::number::le_u32().parse(rest)?
        } else {
            (rest, 0)
        };

        Ok((
            rest,
            BlockHeader {
                crc,
                ty,
                flags,
                size,
                add_size,
            },
        ))
    }
}

#[derive(Debug)]
pub struct Srr {
    pub blocks: Vec<Block>,
}

impl Srr {
    pub fn new(input: &[u8]) -> nom::IResult<&[u8], Self> {
        let mut offset = 0;
        let mut blocks = vec![];
        while offset < input.len() {
            let (rest, header) = BlockHeader::parse(&input[offset..])?;
            let consumed = input[offset..].len() - rest.len();
            println!("{:0>2X?}", &input[offset..(offset + consumed)]);
            println!("{:?} - {}", header, consumed);

            match header.ty {
                BlockType::RarVolumeHeader => {
                    offset += header.size as usize;
                }
                BlockType::RarPackedFile => {
                    offset += 7 + 4;
                    let (rest, block) = RarPackedFile::parse(&input[offset..])?;
                    let consumed = input[offset..].len() - rest.len();
                    println!("{:0>2X?}", &input[offset..(offset + consumed)]);
                    println!(
                        "consumed({:X}) -- header({:X}) -- diff({:X}) -- advancing({:X}) -- index({:X})",
                        consumed,
                        header.size,
                        header.size as usize - consumed,
                        header.size as usize + 7 + 2,
                        offset
                    );
                    // let _unpacked_size = u32::from_le_bytes(input[0..4].try_into().unwrap());
                    // let _os = input[4];
                    // let _file_crc = u32::from_le_bytes(input[5..9].try_into().unwrap());
                    // let _datetime = u32::from_le_bytes(input[9..13].try_into().unwrap());
                    // let _unpack_version = input[13];
                    // let _compression_method = input[14];
                    // let name_length = u16::from_le_bytes(input[15..17].try_into().unwrap());
                    // let _file_attributes = u32::from_le_bytes(input[17..21].try_into().unwrap());

                    // if (header.flags & 0x100) != 0 {
                    //     // let packed_size =
                    //     //     u32::from_le_bytes(input[21..25].try_into().unwrap()) * 0x100000000;
                    //     // let unpacked_size =
                    //     //     u32::from_le_bytes(input[25..29].try_into().unwrap()) * 0x100000000;
                    //     unimplemented!()
                    // }

                    // let untrimmed =
                    //     String::from_utf8_lossy(&input[21..(21 + name_length as usize)]);
                    // let _file_name = match untrimmed.split_once('\0') {
                    //     Some((file_name, _term)) => file_name.to_string(),
                    //     None => untrimmed.to_string(),
                    // };

                    offset += header.size as usize + 7 + 2;
                    blocks.push(Block {
                        header,
                        inner: Some(BlockImpl::RarPackedFile(block)),
                    });
                }
                BlockType::RarOldRecovery => {
                    // untested
                    offset += 7 + 4;
                    let input = &input[offset..];
                    let _packed_size = u32::from_le_bytes(input[0..4].try_into().unwrap());
                    let _rar_version = input[4];
                    let _recovery_sector = u16::from_le_bytes(input[5..7].try_into().unwrap());
                    let _data_sectors = u32::from_le_bytes(input[7..11].try_into().unwrap());
                    offset += 11;
                    blocks.push(Block {
                        header,
                        inner: Some(BlockImpl::RarOldRecovery),
                    });
                }
                BlockType::RarNewSub => todo!(),
                BlockType::SrrHeader => {
                    offset += header.full_size();
                    blocks.push(Block {
                        header,
                        inner: None,
                    });
                }
                BlockType::SrrStoredFile => {
                    offset += 7 + 4;
                    let input = &input[offset..];
                    let block = SrrStoredFile::new(input);
                    offset += 2 + block.file_name.len();
                    let inner = Some(BlockImpl::SrrStoredFile(block));
                    offset += header.add_size as usize;
                    blocks.push(Block { header, inner });
                }
                BlockType::SrrRarFile => {
                    offset += 7;
                    let input = &input[offset..];
                    let block = SrrRarFile::new(input);
                    offset += 2 + block.file_name.len();
                    // let inner = Some(BlockImpl::SrrRarFile(block));
                }
                BlockType::SrrRarPadding => todo!(),
                BlockType::SrrOsoHash
                | BlockType::OldComment
                | BlockType::OldAuthenticity1
                | BlockType::OldSubblock
                | BlockType::OldAuthenticity2
                | BlockType::RarMin
                | BlockType::RarMax => {
                    // won't implement
                    offset += header.size as usize;
                }
            }
        }

        Ok((&[], Self { blocks }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_case_dir() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests")
    }

    fn load_srr(file_name: &str) -> Srr {
        let input = std::fs::read(test_case_dir().join(file_name)).unwrap();
        let (rest, srr) = Srr::new(&input).unwrap();
        assert!(rest.is_empty());
        srr
    }

    #[test]
    fn shanghai_surprise() {
        let srr = load_srr("Shanghai.Surprise.1986.FS.iNTERNAL.DVDRip.x264-REGRET.srr");
        assert_eq!(srr.blocks.len(), 28);
        // println!("{:#?}", srr);
    }

    #[test]
    fn chamber_of_secrets() {
        let _srr =
            load_srr("Harry.Potter.And.The.Chamber.Of.Secrets.2002.DVDRip.XViD-iNTERNAL-TDF.srr");
        // assert_eq!(srr.blocks.len(), 106);
    }

    #[test]
    fn bobs_burgers() {
        let _srr = load_srr("Bobs.Burgers.S02E08.720p.HDTV.X264-DIMENSION.srr");
    }

    #[test]
    fn britney_spears() {
        let _srr = load_srr("Britney_Spears-Stronger-DVDRip-IVTC-SVCD-cHiPs-mVz.srr");
    }

    #[test]
    fn dj_melvin() {
        let _srr = load_srr("DJ_Melvin-L.O.I.S.-CDM-2002-TGX.srr");
    }

    #[test]
    fn nore() {
        let _srr = load_srr("N.O.R.E._-_Nothin-(CDS)-2002-SC.srr");
    }

    #[test]
    fn thickos() {
        let _srr = load_srr("Thickos.scen0r.zine.Issue.01-THiCK0S.srr");
    }
}
