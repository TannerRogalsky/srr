# srr file format

Based primarily on the [.Net implementation here](https://github.com/srrDB/srrcore).

## Blocks

### RarBlock
Represents basic header used in all SRR and RAR blocks.
	
crc             HEAD_CRC: For header CRCs,  RAR calculates a CRC32 and 
                throws out the high-order bytes.
rawtype         HEAD_TYPE
flags           HEAD_FLAGS
_rawdata        All the data (byte string) of this block.
block_position  Offset of the block in the original file/stream.
header_size     The length of the header from this block.

|CRC |TY|FLAG|SIZE[|ADD_SIZE]
Each block begins with the following fields:

HEAD_CRC       2 bytes     CRC of total block or block part
HEAD_TYPE      1 byte      Block type
HEAD_FLAGS     2 bytes     Block flags
HEAD_SIZE      2 bytes     Block size
ADD_SIZE       4 bytes     Optional field - added block size

    Marker block ( MARK_HEAD )
HEAD_CRC        Always 0x6152                    2 bytes
HEAD_TYPE       Header type: 0x72                1 byte
HEAD_FLAGS      Always 0x1a21                    2 bytes
HEAD_SIZE       Block size = 0x0007              2 bytes
    The marker block is actually considered as a fixed byte
sequence: 0x52 0x61 0x72 0x21 0x1a 0x07 0x00

### SrrHeader
Represents marker/srr volume header block.
It contains the name of the ReScene application.

|CRC |TY|FLAG|SIZE|[APNS|APPLICATION NAME...|]
    6969 69 0100
            0000
CRC:    0x6969; (will never be the actual calculated CRC value)
        magic number
TY:     Type: 0x69
FLAG:   0x1 if Application name is present
SIZE:   Header length in bytes (See RAR HEAD_SIZE)
        Name length and name are included in the header, but HL is limited
        to 65535 (0xFFFF) bytes.

APNS:   Application name size. Length of APPLICATION NAME.
        2 bytes. Maximum value: FFF6. Can be 0000 if 0x1 flag is set.
APPLICATION NAME:
        Name of the application, if present. Max 65526 bytes long.

If APNS and name are included in the header, so the maximum possible
APNS value is 0xFFF6 because of SIZE.
    Max application name size:
        FFFF(65535 bytes) (SIZE: 2 bytes) minus:
            - 7 bytes        (HEADER_LENGTH) 
            - 2 bytes        (APNS) 
        65535 - 9 = 65526 (FFF6).
Minimal block used in the old beta 2 equivalent C implementation:
69 69 69 00 00 07 00
Minimal block that ReScene .NET produces when empty string is given.
69 69 69 01 00 07 00 00 00

### SrrStoredFile

SRR block used to store additional files inside the .srr file. e.g. .nfo and .sfv files.
	
|CRC |TY|FLAG| HL | * |  SIZE  | NL |(path)File name|
CRC:    0x6A6A
TY:     Type 0x6A
HL:     Header Length (2 bytes)
FLAG:   0x8000 must always be set for this block to indicate file size
SIZE:   File Size (4 bytes) -> existence indicated in FLAGs
        The maximum file size is 4294967296 bytes (0xFFFFFFFF) or 4096 MiB.
NL: Name Length + path (2 bytes)
    The maximum length of the path + the name is 65522 (0xFFF2).
    The path structure in RAR files is always Windows style: "\" BUT
    ReScene .NET uses the "/" file name separator to store paths. 
    
    Because HL is also 2 bytes, NL can't use the full range.
    0xFFFF - 7 - 4 - 2 = 65522 (0xFFF2)

file_size       The size in bytes of the file stored.
file_name       The name of the file stored after this block.
header_size     The offset in the header where the actual file begins.

### SrrRarFile
We create one SRR block (type 0x71) for each RAR file.
It has a 7 byte header: 2 bytes for file name length, then file name.
Flag 0x1 means recovery records have been removed if present. This
flag is always set in newer versions of ReScene. 

|CRC |TY|FLAG|SIZE| * | NL |RAR File name...|
CRC: 0x7171
TY: Type 0x71
SIZE: Header Length
NL: Name Length of RAR File name (2 bytes)

The maximum length of the path + the name is 65526 (0xFFF6).
    0xFFFF - 7 - 2 = 65526 (0xFFF6)

file_name: The name of the file inside a rar archive.

### SrrOsoHash
SRR block that contains an OpenSubtitles.Org/ISDb hash.
http://trac.opensubtitles.org/projects/opensubtitles/wiki/HashSourceCodes

|CRC |TY|FLAG| HL |  SIZE          |  OSO HASH      | NL |File name|
CRC:    0x6B6B (2 bytes)
TY:     Type 0x6B (1 byte)
FLAG:   no flags (2 bytes)
HL:     Header Length (2 bytes)

SIZE:   File Size (8 bytes)
        The maximum file size is 0xFFFFFFFFFFFFFFFF bytes
OSO HASH: 64bit chksum of the first and last 64k (ISDb hash)
NL: Name Length (2 bytes)
    Because HL is also 2 bytes, NL can't use the full range.
    0xFFFF - 7 - 8 - 8 - 2 = 65510 (0xFFE6)
File name: must match a stored file name

### SrrRarPaddingBlock
Some scene releases, e.g.
    The.Numbers.Station.2013.720p.BluRay.x264-DAA
    Stand.Up.Guys.2012.720p.BluRay.x264-DAA
have padded bytes after the end of the RAR Archive End Block.
This block will include those padded bytes into the SRR file.
    
|CRC |TY|FLAG| HL |  SIZE  |
CRC:    0x6C6C (2 bytes)
TY:     Type 0x6C (1 byte)
FLAG:   Long block (2 bytes)
HL:     Header Length (2 bytes)
        Always 7 + 4 = 11 bytes.

### RarVolumeHeaderBlock
HEAD_CRC    CRC of fields HEAD_TYPE to RESERVED2                  2 bytes
HEAD_TYPE   Header type: 0x73                                     1 byte
HEAD_FLAGS  Bit flags:                                            2 bytes
HEAD_SIZE   Archive header total size including archive comments  2 bytes
RESERVED1   Reserved                                              2 bytes
RESERVED2   Reserved                                              4 bytes

### RarPackedFile
File header (File in archive)
	
"file_name" attribute: File name stored in the block, using the
    backslash (\\) as a directory separator

HEAD_CRC        CRC of fields from HEAD_TYPE to FILEATTR   2 bytes
                and file name
HEAD_TYPE       Header type: 0x74                          1 byte
HEAD_FLAGS      Bit flags                                  2 bytes
HEAD_SIZE       File header full size                      2 bytes
                including file name and comments
                
PACK_SIZE       Compressed file size                       4 bytes
UNP_SIZE        Uncompressed file size                     4 bytes
HOST_OS         Operating system used for archiving        1 byte
FILE_CRC        File CRC                                   4 bytes
FTIME           Date and time in standard MS DOS format    4 bytes
UNP_VER         RAR version needed to extract file         1 byte
                Version number is encoded as
                10 * Major version + minor version.
METHOD          Packing method                             1 byte
NAME_SIZE       File name size                             2 bytes (27-28)
ATTR            File attributes                            4 bytes

HIGH_PACK_SIZE  High 4 bytes of 64 bit value of compressed     4 bytes
                file size. Optional value, presents only if
                bit 0x100 in HEAD_FLAGS is set.
HIGH_UNP_SIZE   High 4 bytes of 64 bit value of uncompressed   4 bytes
                file size. Optional value, presents only
                if bit 0x100 in HEAD_FLAGS is set.
FILE_NAME       File name - string of NAME_SIZE bytes size
SALT            present if (HEAD_FLAGS & 0x400) != 0           8 bytes
EXT_TIME        present if (HEAD_FLAGS & 0x1000) != 0          variable size

other new fields may appear here.

### RarNewSubBlock
**Subclasses** RarPackedFile.

RarNewSubBlock is used for AV, CMT, RR. 
http://stackoverflow.com/questions/8126645/on-which-data-is-the-filecrc-in-newsub-head-of-a-rar-recovery-record-based/
crc = crc32(data, ~0x0fffffff)

### RarEndArchiveBlock
Last block of a RAR file. This block is optional. From rar.exe:
    en    Do not put 'end of archive' block
    
Block with length 26 bytes found! gvd-herorpk1.r40
Hero.Directors.Cut.German.2002.WS.DVDRIP.REPACK.AC3.XviD-GVD
e069 7b 0f40 1a00  ec30ef94  2900  00000000000000  _000000000000_
+FTIME: 2005-07-23 13:22:12
+UNP_VER: Version 2.9 is needed to extract.
+FILE_NAME: AV -> last zeros something to do with this?