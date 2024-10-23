//! Elder Scrolls Master file parser.

#![allow(non_snake_case)]

use chunk_parser::prelude::*;
pub use chunk_parser::Result;
use esm_bindings::fo3::*;

use std::ffi::CString;
use std::io::Read;

use flate2::read::ZlibDecoder;

mod enums;
pub use enums::*;


//------------------------------------------------------------------------------

#[chunk_parser(custom,depth)]
#[deprecated]
pub struct ESMParser {
    localised: bool
}

#[chunk_parser(custom,depth)]
pub struct ESMParser2 {
    localised: bool
}

//------------------------------------------------------------------------------

type RecordParser<P> = fn(parser: &mut P, header: &RecordHeader) -> Result<()>;
type FieldParser<P> = fn(parser: &mut P, header: &FieldHeader) -> Result<()>;

//------------------------------------------------------------------------------

#[derive(Debug)]
pub struct Record {
    pub header: RecordHeader,
    pub fields: Vec<Field>
}

#[derive(Debug)]
pub struct Group {
    pub header: GroupHeader
}

#[derive(Debug)]
pub struct Field {
    pub header: FieldHeader
}

//------------------------------------------------------------------------------

#[derive(Debug)]
pub struct WorldEntry {
    pub world: Record,
    pub world_children: WorldChildren
}

#[derive(Debug)]
pub struct WorldChildren {
    pub cell: Cell,
    pub blocks: Vec<ExteriorCellBlock>
}

#[derive(Debug)]
pub struct ExteriorCellBlock {
    pub sub_blocks: Vec<ExteriorCellSubBlock>
}

#[derive(Debug)]
pub struct ExteriorCellSubBlock {
    pub cells: Vec<Cell>
}

//------------------------------------------------------------------------------

#[derive(Debug)]
pub struct Cell {
    pub cell: Record,
    pub cell_children: Option<CellChildren>
}

#[derive(Debug)]
pub struct CellChildren {
    pub parent_id: u32,
    pub temporary: Option<Vec<Record>>,
    pub persistant: Option<Vec<Record>>
}

#[derive(Debug)]
pub struct CellPersistentChildren {
    pub header: GroupHeader
}

#[derive(Debug)]
pub struct CellTemporaryChildren {
    pub header: GroupHeader
}  


//------------------------------------------------------------------------------

#[derive(Debug)]
pub struct Dialog {
    pub dialog: Record,
    pub children: Option<Group>
}

//------------------------------------------------------------------------------

#[derive(Debug)]
pub struct Quest {
    pub quest: Record,
    pub children: Option<Group>
}

//------------------------------------------------------------------------------

macro_rules! indent {
    ($parser:expr, $($arg:tt)*) => {
        let indent = " ".repeat($parser.depth() as usize * 2);
        print!("{}{}", indent, format!($($arg)*));
    };
}
macro_rules! indentln {
    ($parser:expr, $($arg:tt)*) => {
        let indent = " ".repeat($parser.depth() as usize * 2);
        println!("{}{}", indent, format!($($arg)*));
    };
}

/*
/// Elder Scrolls Master parser implementation.
impl<R> ESMParser<R> where R: std::io::Read + std::io::Seek {
    /// Read a fixed sized string.
    fn read_zstring(&mut self, length: u16) -> Result<CString> {
        let mut v = Vec::with_capacity(length as usize);
        unsafe {
            let ptr = v.as_mut_ptr();
            self.reader().read_exact(std::slice::from_raw_parts_mut(ptr, length as usize))?;
            v.set_len(length as usize);
        }
        Ok(unsafe { CString::from_vec_unchecked(v) })
    }

    /// Read a potentially localised string.
    fn read_lstring(&mut self, length: u16) -> Result<CString> {
        if self.localised { panic!("unimplemented lstring");  }
        else { self.read_zstring(length) }
    }

    /// Decompress a Zlib buffer.
    fn deflate(&mut self, size: usize) -> Result<Vec<u8>> {
        let mut v = Vec::with_capacity(size);
        unsafe {
            let ptr = v.as_mut_ptr();
            self.reader().read_exact(std::slice::from_raw_parts_mut(ptr, size))?;
            v.set_len(size);
        }
        let mut decoder = ZlibDecoder::new(&v[..]);
        let mut decompressed_data = Vec::new();
        decoder.read_to_end(&mut decompressed_data)?;
        Ok(decompressed_data)
    }

    pub fn GRUP(&mut self, header: &RecordHeader) -> Result<()> {
        let RecordHeader { size, type_id, flags, .. } = *header;

//        if self.depth() >= 3 {
//            self.skip(size as u64)?;
//            return Ok(())
//        }

        if type_id == b"GRUP" {
            let GRUP: GroupHeader = unsafe { std::mem::transmute(*header) };
            indentln!(self, "{:?}", GRUP);
        } else {
            if type_id != b"REFR" {
                indentln!(self, "{:?}", header);
            }

            if (flags & 0x00040000) != 0 {
                let _uncompressed_size: u32 = self.read()?;
                let decompressed = &self.deflate(size as usize - 4)?;
                let reader = std::io::Cursor::new(decompressed);
                let mut parser = ESMParser::new(reader);
                *parser.inner_depth() = self.depth();
                parser.localised = self.localised;
                parser.push();
                // this block is for the first compressed record, NPC_
                parser.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, decompressed.len() as u32)?;
                parser.pop();
                return Ok(())
            }
        }

        match &type_id.0 {
            b"GLOB" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FNAM" => {
                            let FNAM: u8 = parser.read()?;
                            println!("{:?}", FNAM);
                        },
                        b"FLTV" => {
                            let FLTV: f32 = parser.read()?;
                            println!("{:?}", FLTV);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"FACT" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"XNAM" => {
                            let XNAM: XNAM = parser.read()?;
                            println!("{:?}", XNAM);
                        },
                        b"DATA" => {
                            let DATA: u32 = parser.read()?;
                            println!("{:#010x}", DATA);
                        },
                        b"RNAM" => {
                            let RNAM: u32 = parser.read()?;
                            println!("{:#010x}", RNAM);
                        },
                        b"MNAM" => {
                            let MNAM = parser.read_lstring(header.size)?;
                            println!("{:?}", MNAM);
                        },
                        b"FNAM" => {
                            let FNAM = parser.read_lstring(header.size)?;
                            println!("{:?}", FNAM);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"TXST" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        tx if tx >= b"TX00" && tx <= b"TX07" => {
                            let TX = parser.read_zstring(header.size)?;
                            println!("{:?}", TX);
                        },
                        b"DNAM" => {
                            let DNAM: u16 = parser.read()?;
                            println!("{:?}", DNAM);
                        },
                        /*b"DODT" => {
                            let DODT: DODT = parser.read()?;
                            println!("{:?}", DODT);
                        },*/
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!(" Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"CLAS" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, " {:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"DESC" => {
                            let DESC = parser.read_lstring(header.size)?;
                            println!("{:?}", DESC);
                        },
                        /*b"ICON" => {
                            let ICON = parser.read_zstring(header.size)?;
                            println!("{:?}", ICON);
                        },*/
                        b"DATA" => {
                            let CLAS: CLAS = parser.read()?;
                            println!("{:?}", CLAS);
                        },
                        b"ATTR" => {
                            let ATTR: ATTR = parser.read()?;
                            println!("{:?}", ATTR);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"SOUN" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"FNAM" => {
                            let FNAM = parser.read_zstring(header.size)?;
                            println!("{:?}", FNAM);
                        },
                        b"SNDD" => {
                            let SNDD: SNDD = parser.read()?;
                            println!("{:?}", SNDD);
                        },
                        b"SDSC" => {
                            let SDSC: formid_t = parser.read()?;
                            println!("{:?}", SDSC);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"ASPC" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"SNAM" => {
                            let SNAM: formid_t = parser.read()?;
                            println!("{:?}", SNAM);
                        },
                        b"RDAT" => {
                            let RDAT: formid_t = parser.read()?;
                            println!("{:?}", RDAT);
                        },
                        b"BNAM" => {
                            let BNAM: formid_t = parser.read()?;
                            println!("{:?}", BNAM);
                        },
                        b"ANAM" => {
                            let ANAM: [u8;4] = parser.read()?;
                            println!("ANAM {{ unknown: {:?} }}", ANAM);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"MGEF" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        //b"VMAD" => {},
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"DESC" => {
                            let DESC = parser.read_lstring(header.size)?;
                            println!("{:?}", DESC);
                        },
                        /*b"MDOB" => {
                            let MDOB: formid_t = parser.read()?;
                            println!("{:?}", MDOB);
                        },
                        b"KSIZ" => {
                            let KSIZ: u32 = parser.read()?;
                            println!("{:?}", KSIZ);
                        },
                        b"KWDA" => {},*/
                        b"DATA" => {
                            let MGEF: MGEF = parser.read()?;
                            println!("{:?}", MGEF);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"ENCH" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"ENIT" => {
                            let ENIT: ENIT = parser.read()?;
                            println!("{:?}", ENIT);
                        },
                        b"EFID" => {
                            let EFID: formid_t = parser.read()?;
                            println!("{:?}", EFID);
                        },
                        b"EFIT" => {
                            let EFIT: EFIT = parser.read()?;
                            println!("{:?}", EFIT);
                        },
                        /*b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },*/
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"SCPT" => {
                let SCPT = self.read_lstring(size as u16)?;
                println!("{:?}", SCPT);
            },
            b"SPEL" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"SPIT" => {
                            let SPIT: SPIT = parser.read()?;
                            println!("{:?}", SPIT);
                        },
                        b"EFID" => {
                            let EFID: formid_t = parser.read()?;
                            println!("{:?}", EFID);
                        },
                        b"EFIT" => {
                            let EFIT: EFIT = parser.read()?;
                            println!("{:?}", EFIT);
                        },
                        b"CTDA" => {
                            let CTDA: CTDA = parser.read()?;
                            println!("{:?}", CTDA);
                        },
                        /*b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },*/
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"ACTI" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"SCRI" => {
                            let SCRI: formid_t = parser.read()?;
                            println!("{:?}", SCRI);
                        },
                        b"VNAM" => {
                            let VNAM: formid_t = parser.read()?;
                            println!("{:?}", VNAM);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"SNAM" => {
                            let SNAM: formid_t = parser.read()?;
                            println!("{:?}", SNAM);
                        },
                        b"DEST" => {
                            let DEST: [u8;8] = parser.read()?;
                            println!("DEST {{ unknown: {:?} }}", DEST);
                        },
                        b"DSTD" => {
                            let DSTD: DSTD = parser.read()?;
                            println!("{:?}", DSTD);
                        },
                        b"DSTF" => {
                            println!();
                        },
                        b"DMDL" => {
                            let DMDL = parser.read_zstring(header.size)?;
                            println!("{:?}", DMDL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"TERM" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"DESC" => {
                            let DESC = parser.read_lstring(header.size)?;
                            println!("{:?}", DESC);
                        },
                        b"CTDA" => {
                            let CTDA: CTDA = parser.read()?;
                            println!("{:?}", CTDA);
                        },
                        b"RNAM" => {
                            let RNAM = parser.read_zstring(header.size)?;
                            println!("{:?}", RNAM);
                        },
                        b"ITXT" => {
                            let ITXT = parser.read_zstring(header.size)?;
                            println!("{:?}", ITXT);
                        },
                        b"SNAM" => {
                            let SNAM: formid_t = parser.read()?;
                            println!("{:?}", SNAM);
                        },
                        b"SCHR" => {
                            let SCHR: SCHR = parser.read()?;
                            println!("{:?}", SCHR);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"CONT" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"SCRI" => {
                            let SCRI: formid_t = parser.read()?;
                            println!("{:?}", SCRI);
                        },
                        b"DATA" => {
                            let DATA: [u8;5] = parser.read()?;
                            println!("DATA {{ unknown: {:?} }}", DATA);
                        },
                        b"CNTO" => {
                            let CNTO: CNTO = parser.read()?;
                            println!("{:?}", CNTO);
                        },
                        b"COED" => {
                            let COED: COED = parser.read()?;
                            println!("{:?}", COED);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"LIGH" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"SCRI" => {
                            let SCRI: formid_t = parser.read()?;
                            println!("{:?}", SCRI);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"MISC" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"ICON" => {
                            let ICON = parser.read_zstring(header.size)?;
                            println!("{:?}", ICON);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"STAT" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"MSTT" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"DATA" => {
                            let DATA: u8 = parser.read()?;
                            println!("{:?}", DATA);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"PWAT" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"FURN" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"MNAM" => {
                            let MNAM: u32 = parser.read()?;
                            println!("{:?}", MNAM);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"WEAP" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"MOD2" => {
                            let MOD2 = parser.read_zstring(header.size)?;
                            println!("{:?}", MOD2);
                        },
                        b"MOD3" => {
                            let MOD3 = parser.read_zstring(header.size)?;
                            println!("{:?}", MOD3);
                        },
                        b"MOD4" => {
                            let MOD4 = parser.read_zstring(header.size)?;
                            println!("{:?}", MOD4);
                        },
                        b"ICON" => {
                            let ICON = parser.read_zstring(header.size)?;
                            println!("{:?}", ICON);
                        },
                        b"CRDT" => {
                            let CRDT: CRDT = parser.read()?;
                            println!("{:?}", CRDT);
                        },
                        b"EITM" => {
                            let EITM: formid_t = parser.read()?;
                            println!("{:?}", EITM);
                        },
                        b"ETYP" => {
                            let ETYP: u32 = parser.read()?;
                            println!("{:?}", ETYP);
                        },
                        b"DATA" => {
                            let DATA: DATA = parser.read()?;
                            println!("{:?}", DATA);
                        },
                        b"REPL" => {
                            let REPL: formid_t = parser.read()?;
                            println!("{:?}", REPL);
                        },
                        b"SCRI" => {
                            let SCRI: formid_t = parser.read()?;
                            println!("{:?}", SCRI);
                        },
                        b"NAM0" => {
                            let NAM0: formid_t = parser.read()?;
                            println!("{:?}", NAM0);
                        },
                        b"NAM6" => {
                            let NAM6: formid_t = parser.read()?;
                            println!("{:?}", NAM6);
                        },
                        b"NAM8" => {
                            let NAM8: formid_t = parser.read()?;
                            println!("{:?}", NAM8);
                        },
                        b"NAM9" => {
                            let NAM9: formid_t = parser.read()?;
                            println!("{:?}", NAM9);
                        },
                        b"DNAM" => {
                            let DNAM: DNAM = parser.read()?;
                            println!("{:?}", DNAM);
                        },
                        b"INAM" => {
                            let INAM: formid_t = parser.read()?;
                            println!("{:?}", INAM);
                        },
                        b"NNAM" => {
                            let NNAM = parser.read_zstring(header.size)?;
                            println!("{:?}", NNAM);
                        },
                        b"SNAM" => {
                            let SNAM: formid_t = parser.read()?;
                            println!("{:?}", SNAM);
                        },
                        b"TNAM" => {
                            let TNAM: formid_t = parser.read()?;
                            println!("{:?}", TNAM);
                        },
                        b"UNAM" => {
                            let UNAM: formid_t = parser.read()?;
                            println!("{:?}", UNAM);
                        },
                        b"VNAM" => {
                            let VNAM: u32 = parser.read()?;
                            println!("{:?}", VNAM);
                        },
                        b"WNAM" => {
                            let WNAM: formid_t = parser.read()?;
                            println!("{:?}", WNAM);
                        },
                        b"XNAM" => {
                            let XNAM: formid_t = parser.read()?;
                            println!("{:?}", XNAM);
                        },
                        b"YNAM" => {
                            let YNAM: formid_t = parser.read()?;
                            println!("{:?}", YNAM);
                        },
                        b"ZNAM" => {
                            let ZNAM: formid_t = parser.read()?;
                            println!("{:?}", ZNAM);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"AMMO" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"ICON" => {
                            let ICON = parser.read_zstring(header.size)?;
                            println!("{:?}", ICON);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"CREA" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"LVLC" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"ALCH" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"ICON" => {
                            let ICON = parser.read_zstring(header.size)?;
                            println!("{:?}", ICON);
                        },
                        b"EFID" => {
                            let EFID: formid_t = parser.read()?;
                            println!("{:?}", EFID);
                        },
                        b"EFIT" => {
                            let EFIT: EFIT = parser.read()?;
                            println!("{:?}", EFIT);
                        },
                        b"CTDA" => {
                            let CTDA: CTDA = parser.read()?;
                            println!("{:?}", CTDA);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"NOTE" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"ICON" => {
                            let ICON = parser.read_zstring(header.size)?;
                            println!("{:?}", ICON);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"PROJ" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"LVLI" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"REGN" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"NAVI" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"DIAL" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"QUST" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"IDLE" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"PACK" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"CSTY" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"ANIO" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"WATR" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"EFSH" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"ICON" => {
                            let ICON = parser.read_zstring(header.size)?;
                            println!("{:?}", ICON);
                        },
                        b"ICO2" => {
                            let ICO2 = parser.read_zstring(header.size)?;
                            println!("{:?}", ICO2);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"EXPL" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"DEBR" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"IMGS" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"FLST" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"PERK" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"DESC" => {
                            let DESC = parser.read_lstring(header.size)?;
                            println!("{:?}", DESC);
                        },
                        b"ICON" => {
                            let ICON = parser.read_zstring(header.size)?;
                            println!("{:?}", ICON);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"BPTD" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"ADDN" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"CAMS" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"CPTH" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"CTDA" => {
                            let CTDA: CTDA = parser.read()?;
                            println!("{:?}", CTDA);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"VTYP" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"IPCT" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"IPDS" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"ARMA" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"MOD3" => {
                            let MOD3 = parser.read_zstring(header.size)?;
                            println!("{:?}", MOD3);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"ECZN" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"MESG" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"DESC" => {
                            let DESC = parser.read_lstring(header.size)?;
                            println!("{:?}", DESC);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"RGDL" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"NPC_" => { self.skip(size as u64)?; },
            b"WRLD" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"CNAM" => {
                            let CNAM: formid_t = parser.read()?;
                            println!("{:?}", CNAM);
                        },
                        b"XXXX" => {
                            let XXXX_size: u32 = parser.read()?;
                            println!();
                            let next: FieldHeader = parser.read()?;
                            indentln!(parser, "{:?}", next);
                            parser.skip(XXXX_size as u64)?;
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"TACT" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"VNAM" => {
                            let VNAM: formid_t = parser.read()?;
                            println!("{:?}", VNAM);
                        },
                        b"SCRI" => {
                            let SCRI: formid_t = parser.read()?;
                            println!("{:?}", SCRI);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"ARMO" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"EITM" => {
                            let EITM: formid_t = parser.read()?;
                            println!("{:?}", EITM);
                        },
                        b"ICON" => {
                            let ICON = parser.read_zstring(header.size)?;
                            println!("{:?}", ICON);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"MODS" => {
                            let MODS = parser.read_zstring(header.size)?;
                            println!("{:?}", MODS);
                        },
                        b"MOD2" => {
                            let MOD2 = parser.read_zstring(header.size)?;
                            println!("{:?}", MOD2);
                        },
                        b"MOD3" => {
                            let MOD3 = parser.read_zstring(header.size)?;
                            println!("{:?}", MOD3);
                        },
                        b"MO2S" => {
                            let MO2S = parser.read_zstring(header.size)?;
                            println!("{:?}", MO2S);
                        },
                        b"MO3S" => {
                            let MO3S = parser.read_zstring(header.size)?;
                            println!("{:?}", MO3S);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"DOOR" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"SCRI" => {
                            let SCRI: formid_t = parser.read()?;
                            println!("{:?}", SCRI);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"SCOL" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"IDLM" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"IMAD" => { self.skip(size as u64)?; },
            b"GRUP" => {
                self.parse_records(ESMParser::GRUP, header.size as u64)?;
                return Ok(())
            },
            b"CELL" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"ACRE" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"NAVM" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"ACHR" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"INFO" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"REFR" => { self.skip(size as u64)?; },
            _ => {
                self.skip(size as u64)?;
                println!("Unknown record '{}'", type_id);
            },
        }
        Ok(())
    }

    pub fn TES4(&mut self, header: &RecordHeader) -> Result<()> {
        let RecordHeader { type_id, size, .. } = *header;
        match &type_id.0 {
            b"TES4" => {
                println!("{:?} ", header);

                let flags = header.flags;
                if (flags & 0x00000001) != 0 {} // Master (ESM) file
                self.localised = (flags & 0x00000080) != 0;
                if (flags & 0x00000200) != 0 {} // Light Master (ESL) File

                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"HEDR" => {
                            let HEDR: HEDR = parser.read()?;
                            println!("{:?}", HEDR);
                        },
                        b"CNAM" => {
                            let CNAM = parser.read_zstring(header.size)?;
                            println!("{:?}", CNAM);
                        },
                        b"MAST" => {
                            let MAST = parser.read_zstring(header.size)?;
                            println!("{:?}", MAST);
                        },
                        b"DATA" => {
                            let DATA: u64 = parser.read()?;
                            println!("{:?}", DATA);
                        },
                        b"ONAM" => {
                            parser.skip(header.size as u64)?;
                            println!("Unimplemented");
                        },
                        _ => { println!("Unknown typeid '{}'", header.type_id) }
                    }
                    Ok(())
                }, size)?;
            },
            b"GRUP" => {
                let GRUP: GroupHeader = unsafe { std::mem::transmute(*header) };
                println!("{:?}", GRUP);
                self.parse_records(ESMParser::GRUP, size as u64)?;
                return Ok(())
            },
            _ => { println!("Unknown Top Record: '{}'", type_id)}
        }
        Ok(())
    }

    pub fn parse_fields(&mut self, f: FieldParser<Self>, total_size: u32) -> Result<()> {
        if total_size == 0 { return Ok(()) }
        let loop_end = self.reader().stream_position()? + total_size as u64;
        self.push();
        match loop {
            let header: FieldHeader = self.read()?;
            let start = self.reader().stream_position()?;
            let size = header.size as u64;
            f(self, &header)?; // parse the contents
            let end = start + size;
            let pos = self.reader().stream_position()?;
            if pos == loop_end { break Ok(()) } // function consumed chunk
            else if pos != end { return Err(chunk_parser::Error::ParseError) } // function made a mistake
        } {
            res => { self.pop(); res }
        }
    }

    fn parse_records(&mut self, f: RecordParser<Self>, total_size: u64) -> Result<()> {
        if total_size == 0 { return Ok(()) }
        let loop_end = self.reader().stream_position()? + total_size as u64;
        self.push();
        match loop {
            let start = self.reader().stream_position()?;
            let mut header: RecordHeader = self.read()?;
            let mut size = header.size as u64;
            if header.type_id != b"GRUP" { size += 24; }
            else { header.size -= 24; }
            f(self, &header)?; // parse the contents
            let end = start + size;
            let pos = self.reader().stream_position()?;
            if pos == loop_end { break Ok(()) } // function consumed chunk
            else if pos != end { return Err(chunk_parser::Error::ParseError) } // function made a mistake
        } {
            res => { self.pop(); res }
        }
    }

    pub fn parse_top_level(&mut self, f: RecordParser<Self>) -> Result<()> {
        let total_size = self.reader().seek(std::io::SeekFrom::End(0))?;
        self.reader().seek(std::io::SeekFrom::Start(0))?;
//        self.pop();
        self.parse_records(f, total_size)?;
//        self.push();
        Ok(())
    }
}
 */

//------------------------------------------------------------------------------


impl<R> ESMParser2<R> where R: std::io::Read + std::io::Seek {
    
    pub fn parse_until<F>(&mut self, limit: u64, f: fn (&mut Self) -> Result<F>) -> Result<Vec<F>> {
        let mut out = Vec::new();
        while self.reader.stream_position()? < limit {
            out.push(f(self)?);
        }
        Ok(out)
    }

    pub fn parse_top_level(&mut self) -> Result<()> {

        // Get file size
        let total_size = self.reader().seek(std::io::SeekFrom::End(0))?;

        // Go to start of file
        self.reader().seek(std::io::SeekFrom::Start(0))?;

        // Parse the FileHeader record
        let _header_record = self.parse_record()?;

        // Parse the rest of the record groups
        self.parse_until(total_size, Self::parse_top_group)?;

        Ok(())
    }

    pub fn parse_record(&mut self) -> Result<Record> {
        let header: RecordHeader = self.read()?;
        indentln!(self, "{:?}", header);
        if header.type_id == b"GRUP" {
            panic!("Unexpected GRUP record: {:?}", header);
        }
        
        // TODO handle decrompression
        if header.flags & 0x40000000 != 0 {
            self.skip(header.size as u64)?;
        } else {
            match header.type_id.0 {

                _ => {
                    self.skip(header.size as u64)?;
                }
            }
    
        }
        
        
        Ok(Record { header, fields: Vec::new() })
    }

    pub fn parse_subgroup_record(&mut self) -> Result<Record> {
        let header: RecordHeader = self.read()?;
        indentln!(self, "{:?}", header);
        match &header.type_id.0 {
            b"REFR" | b"ACHR" | b"PHZD" | b"LAND" | b"NAVM" | b"PGRE" | b"PMIS" => {
                // TODO handle decrompression
                if header.flags & 0x40000000 != 0 {
                    self.skip(header.size as u64)?;
                } else {
                    match header.type_id.0 {

                        _ => {
                            self.skip(header.size as u64)?;
                        }
                    }
            
                }
                
                
                Ok(Record { header, fields: Vec::new() })
            }
            _ => {
                panic!("Unexpected record type: {:?}", header);
            }
        }
        
        
    }

    pub fn parse_group(&mut self) -> Result<Group> {
        let header: GroupHeader = self.read()?;
        let content_size = header.size as u64 - 24;
        let content_end = self.reader().stream_position()? + content_size;

        let label = header.get_label();

        if header.type_id != b"GRUP" {
            panic!("Expected GRUP record got: {:?} instead", header.type_id);
        }

        match label {
            // Top Group
            GroupLabel::Top(type_id) => {
                match &type_id.0 {
                    b"WRLD" => {
                        indentln!(self, "{:?}", label);
                        self.push();
                        self.parse_until(content_end, Self::parse_world_entry)?;
                        self.pop();
                        
                    }
                    b"CELL" => {
                        self.push();
                        self.parse_until(content_end, Self::parse_group)?;
                        self.pop();
                    }
                    b"QUST" => {
                        self.push();
                        self.skip(header.size as u64 - 24)?;
                        self.pop();
                    }
                    b"DIAL" => {
                        indentln!(self, "{:?}", header);
                        self.push();
                        //self.skip(header.size as u64 - 24)?;
                        while self.reader().stream_position()? < content_end {
                            let _dialog = self.parse_record()?;

                            //indentln!(self, "{:?}", dialog);

                            let next_header: GroupHeader = self.read()?;
                            self.reader.seek_relative(-24)?;

                            if &next_header.type_id.0 == b"GRUP" && next_header.group_type == GroupLabelType_TopicChildren {
                                self.push();
                                let _children = self.parse_group()?;
                                self.pop();
                            }

                        }
                        self.pop();
                    }
                    _ => {
                        indentln!(self, "{:?}", label);
                        self.push();
                        // Attempt to parse the non-custom groups which are just a list of records
                        //let _records = self.parse_records(content_size)?;
                        let _records = self.parse_until(content_end, Self::parse_record)?;
                        self.pop();
                    }
                }
            }
            // World Children
            GroupLabel::WorldChildren(_) => {
                panic!("WorldChildren is handled by another function, you should not see this.");
            }
            // Interior Cell Block
            GroupLabel::InteriorCellBlock(_index) => {
                self.skip(content_size)?;
            }
            // Interior Cell Sub-Block
            GroupLabel::InteriorCellSubBlock(_index) => {
                self.skip(content_size)?;
            }
            // Exterior Cell Block
            GroupLabel::ExteriorCellBlock(coords) => {
                indentln!(self, "Exterior Cell Block({:?})", coords);
                self.push();
                self.parse_until(content_end, Self::parse_group)?;
                self.pop();
            }
            // Exterior Cell Sub-block
            GroupLabel::ExteriorCellSubBlock(coords) => {
                indentln!(self, "Exterior Cell Sub-Block({:?})", coords);
                self.push();
                let _cell = self.parse_until(content_end, Self::parse_cell)?;
                self.pop();
                // let next_id: GroupHeader = self.read()?;
                // self.reader.seek_relative(-24)?;
                // indentln!(self, "Next ID {:?}", next_id.try_get_label().unwrap());
                //self.skip(size)?;
            }
            // Cell Children
            GroupLabel::CellChildren(record_id) => {
                indentln!(self, "Cell Children({:?})", record_id);
                self.skip(content_size)?;
            }
            // Topic Children
            GroupLabel::TopicChildren(_record_id) => {
                //indentln!(self, "Topic Children({:?})", record_id);
                self.skip(content_size)?;
            }
            // Cell Persistent Children
            GroupLabel::CellPersistentChildren(_record_id) => {
                self.skip(content_size)?;
            }
            // Cell Temporary Children
            GroupLabel::CellTemporaryChildren(_record_id) => {
                self.skip(content_size)?;
            }
            // Cell Visible Distant Children
            GroupLabel::CellVisibleDistantChildren(_record_id) => {
                self.skip(content_size)?;
            }
            GroupLabel::Unknown(_type_id) => {
                self.skip(content_size)?;
            }
        }

        

        Ok(Group { header })
    }

    pub fn parse_top_group(&mut self) -> Result<TopGroup> {
        let header: GroupHeader = self.read()?;
        let limit = self.reader.stream_position()? + header.size as u64 - 24;

        println!("{:?} --------------------------------------------------", header.get_label());
        
        self.push();

        let records;

        match header.get_label() {
            GroupLabel::Top(id) => {
                match &id.0 {
                    
                    b"AACT" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"ACTI" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"ADDN" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"AECH" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"ALCH" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"AMDL" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"AMMO" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"ANIO" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"AORU" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"ARMA" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"ARMO" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"ARTO" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"ASPC" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"ASTP" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"AVIF" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"BNDS" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"BOOK" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"BPTD" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"CAMS" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"CELL" => { records = TopGroup::Cell(self.parse_until(limit, Self::parse_group)?); }
                    b"CLAS" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"CLFM" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"CLMT" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"CMPO" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"COBJ" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"COLL" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"CONT" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"CPTH" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"CSTY" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"DEBR" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"DFOB" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"DLVW" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"DMGT" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"DOBJ" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"DOOR" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"ECZN" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"EFSH" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"ENCH" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"EQUP" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"EXPL" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"FACT" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"FLOR" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"FLST" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"FSTP" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"FSTS" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"FURN" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"GDRY" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"GLOB" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"GMST" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"GRAS" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"HAZD" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"HDPT" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"IDLE" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"IDLM" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"IMAD" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"IMGS" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"INGR" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"INNR" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"IPCT" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"IPDS" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"KEYM" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"KSSM" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"KYWD" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"LAYR" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"LCRT" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"LCTN" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"LENS" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"LGTM" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"LIGH" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"LSCR" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"LTEX" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"LVLI" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"LVLN" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"MATO" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"MATT" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"MESG" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"MGEF" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"MISC" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"MOVT" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"MSTT" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"MSWP" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"MUSC" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"MUST" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"NAVI" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"NOCM" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"NOTE" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"NPC_" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"OMOD" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"OTFT" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"OVIS" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"PACK" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"PERK" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"PKIN" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"PROJ" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    //b"QUST" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"RACE" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"REGN" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"RELA" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"REVB" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"RFCT" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"RFGP" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"SCCO" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"SCOL" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"SCSN" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"SMBN" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"SMEN" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"SMQN" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"SNCT" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"SNDR" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"SOPM" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"SOUN" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"SPEL" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"SPGD" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"STAG" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"STAT" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"TACT" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"TERM" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"TREE" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"TRNS" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"TXST" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"VTYP" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"WATR" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"WEAP" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"WRLD" => { records = TopGroup::Worldspace(self.parse_until(limit, Self::parse_world_entry)?); }
                    b"WTHR" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }
                    b"ZOOM" => { records = TopGroup::Unhandled(self.parse_until(limit, Self::parse_record)?); }

                    _ => {
                        self.seek(limit)?;
                        indentln!(self, "Skipped");
                        records = TopGroup::Skipped;
                    }
                    
                }
            },
            _ => panic!("Expected top group, got: {:?}", header)
        }

        self.pop();
        Ok(records)
    }

    pub fn parse_cell(&mut self) -> Result<Cell> {
        let cell = self.parse_record()?;
        let mut cell_children = None;
        indentln!(self, "{:?}", cell.header);

        let next_header: GroupHeader = self.read()?;
        self.rewind(24)?;

        if next_header.type_id == b"GRUP" && next_header.group_type == GroupLabelType_CellChildren {
            if let Ok(cc) = self.parse_cell_children() {
                cell_children = Some(cc);
            } else {
                panic!("Could not parse CellChildren");
            }
        }

        Ok(Cell { cell, cell_children })
    }

    pub fn parse_cell_children(&mut self) -> Result<CellChildren> {
        self.push();
        let header: GroupHeader = self.read()?;
        let label = header.get_label();
        
        

        match label {
            GroupLabel::CellChildren(parent_id) => {
                indentln!(self, "{:?}", label);
                let mut temporary = None;
                let mut persistant = None;

                let next_header: GroupHeader = self.read()?;
                let next_label = next_header.get_label();
                let next_limit = self.reader.stream_position()? + next_header.size as u64 - 24;

                self.push();

                indentln!(self, "{:?}", next_label);

                match next_label {
                    GroupLabel::CellPersistentChildren(_parent) => {
                        persistant = Some(self.parse_until(next_limit, Self::parse_subgroup_record)?);
                    }
                    GroupLabel::CellTemporaryChildren(_parent) => {
                        temporary = Some(self.parse_until(next_limit, Self::parse_subgroup_record)?);
                    }
                    _ => {
                        // If CellChildren exists, there should be at least one sub group
                        // Panic if the sub group is not correct
                        panic!("Unexpected group type encountered inside CellChildren: {:?}", header);
                    }
                }


                let next_header: GroupHeader = self.read()?;

                // Rewind if next group is not actually a group
                if &next_header.type_id.0 != b"GRUP" {
                    self.rewind(24)?;
                    
                    let out  = CellChildren { parent_id, temporary, persistant };
                    //indentln!(self, "{:?}", out);
                    self.pop();
                    self.pop();
                    Ok(out)

                
                } else {
                    let next_label = next_header.get_label();
                    let next_limit = self.reader().stream_position()? + next_header.size as u64 - 24;

                    match next_label {
                        GroupLabel::CellTemporaryChildren(_parent_id) => {
                            indentln!(self, "{:?}", next_label);
                            //self.skip(next_header.size as u64 - 24)?;
                            temporary = Some(self.parse_until(next_limit, Self::parse_subgroup_record)?);
                        }
                        GroupLabel::CellPersistentChildren(_parent_id) => {
                            indentln!(self, "{:?}", next_label);
                            //self.skip(next_header.size as u64 - 24)?;
                            persistant = Some(self.parse_until(next_limit, Self::parse_subgroup_record)?);
                        }
                        _ => {
                            // Next group does not belong to cell children, rewind and continue
                            self.rewind(24)?;
                        }
                    }

                    let out = CellChildren { parent_id, temporary, persistant };
                    //indentln!(self, "{:?}", out);
                    self.pop();
                    self.pop();
                    Ok(out)
                }

                
            }
            _ => {
                panic!("Tried to parse CellChildren, got different group type: {:?}", header);
            }
        }

    }

    pub fn parse_world_entry(&mut self) -> Result<WorldEntry> {

        let world = self.parse_record()?;
        indentln!(self, "{:?}", world.header);

        let world_children = self.parse_world_children()?;

        Ok(WorldEntry { world, world_children })
    }

    pub fn parse_world_children(&mut self) -> Result<WorldChildren> {
        self.push();
        let header: GroupHeader = self.read()?;
        let limit = self.reader.stream_position()? + header.size as u64 - 24;
        let label = header.get_label();
        
        indentln!(self, "{:?}", label);
        self.push();
        let cell = self.parse_cell()?;

        // TODO Push correct values to vector
        let blocks = Vec::new();

        self.pop();
        let _blocks = self.parse_until(limit, Self::parse_group)?;

        self.pop();
        Ok(WorldChildren { cell, blocks })
    }

    pub fn parse_fields(&mut self, f: FieldParser<Self>, total_size: u32) -> Result<()> {
        if total_size == 0 { return Ok(()) }
        let loop_end = self.reader().stream_position()? + total_size as u64;
        self.push();
        match loop {
            let header: FieldHeader = self.read()?;
            let start = self.reader().stream_position()?;
            let size = header.size as u64;
            f(self, &header)?; // parse the contents
            let end = start + size;
            let pos = self.reader().stream_position()?;
            if pos == loop_end { break Ok(()) } // function consumed chunk
            else if pos != end { return Err(chunk_parser::Error::ParseError) } // function made a mistake
        } {
            res => { self.pop(); res }
        }
    }

    pub fn parse_dial(&mut self) -> Result<Dialog> {
        todo!()
    }

    pub fn parse_qust(&mut self) -> Result<Quest> {
        todo!()
    }
}

//------------------------------------------------------------------------------

pub mod prelude {
    pub use chunk_parser::prelude::*;
    pub use {super::ESMParser, super::ESMParser2};
}

//==============================================================================

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn zeta() -> chunk_parser::Result<()> {
        const DATA: &[u8] = include_bytes!("../data/Zeta.esm");
        let mut esm = ESMParser2::cursor(DATA);
        esm.parse_top_level()
    }

    //#[test]
    fn fallout4() -> chunk_parser::Result<()> {
        const DATA: &[u8] = include_bytes!("../data/Fallout4.esm");
        let mut esm = ESMParser2::cursor(DATA);
        esm.parse_top_level()
    }
}
