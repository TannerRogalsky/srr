mod blocks;

pub use blocks::*;
use nom::Parser as _;

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
        fn parse_block_type(b: &[u8]) -> nom::IResult<&[u8], BlockType> {
            match BlockType::try_from(b[0]) {
                Ok(ty) => Ok((b, ty)),
                Err(_err) => Err(nom::Err::Error(nom::error_position!(
                    b,
                    nom::error::ErrorKind::Tag
                ))),
            }
        }

        let (rest, crc) = nom::number::le_u16().parse(input)?;
        let (rest, ty) = nom::bytes::take(1usize)
            .and_then(parse_block_type)
            .parse(rest)?;
        let (rest, flags) = nom::number::le_u16().parse(rest)?;
        let (rest, size) = nom::number::le_u16().parse(rest)?;

        let has_add_size =
            (flags & 0x8000) > 0 || matches!(ty, BlockType::RarPackedFile | BlockType::RarNewSub);
        let (rest, add_size) = nom::combinator::cond(has_add_size, nom::number::le_u32())
            .map(|add_size| add_size.unwrap_or(0))
            .parse(rest)?;

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

            match header.ty {
                BlockType::RarVolumeHeader => {
                    offset += header.size as usize;
                }
                BlockType::RarPackedFile => {
                    offset += consumed;
                    let size = header.size as usize - consumed;
                    let (_rest, block) = RarPackedFile::parse(&input[offset..][..size], &header)?;
                    offset += size;
                    blocks.push(Block {
                        header,
                        inner: Some(BlockImpl::RarPackedFile(block)),
                    });
                }
                BlockType::RarOldRecovery => {
                    // untested
                    offset += consumed;
                    let (rest, block) = RarOldRecovery::parse(&input[offset..])?;
                    offset += input[offset..].len() - rest.len();
                    blocks.push(Block {
                        header,
                        inner: Some(BlockImpl::RarOldRecovery(block)),
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
                    offset += consumed;
                    let size = header.size as usize - consumed;
                    let (_rest, block) = SrrStoredFile::new(&input[offset..][..size])?;
                    offset += size + header.add_size as usize;
                    blocks.push(Block {
                        header,
                        inner: Some(BlockImpl::SrrStoredFile(block)),
                    });
                }
                BlockType::SrrRarFile => {
                    offset += consumed;
                    let (rest, block) = SrrRarFile::new(&input[offset..])?;
                    let consumed = input[offset..].len() - rest.len();
                    offset += consumed;
                    blocks.push(Block {
                        header,
                        inner: Some(BlockImpl::SrrRarFile(block)),
                    })
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

        Ok((&input[offset..], Self { blocks }))
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
        let _srr = load_srr("Shanghai.Surprise.1986.FS.iNTERNAL.DVDRip.x264-REGRET.srr");
        // assert_eq!(srr.blocks.len(), 28);
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
