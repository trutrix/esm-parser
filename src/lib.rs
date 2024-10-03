//! Elder Scrolls Mod file parser.

use chunk_parser::prelude::*;
use esm_bindings::*;
use std::ffi::CString;

//------------------------------------------------------------------------------

#[chunk_parser]
pub struct ESMParser {
    localised: bool
}

/// Elder Scrolls Mod parser implementation.
impl<R> Parser for ESMParser<R> where R: std::io::Read + std::io::Seek {
    type Header = (TypeId, u32);
    type Size = u32;

    fn read_header(&mut self) -> chunk_parser::Result<Self::Header> {
        Ok((self.read()?, self.read::<u32>()? + 16))
    }

    fn guesser(&mut self, ( typeid, size ): &Self::Header) -> chunk_parser::Result<u32> {
        let pos = self.position()?;
        let depth = self.depth();

        println!(
            "{:#08} {}{} {}{: >16} bytes", self.position()? - 8,
            " ".repeat(depth * 2), FourCC(*typeid),
            " ".repeat(16 - depth * 2), size
        );

        // if the next 4 bytes are a valid fourcc, it could be a container like FORM
        let subid = self.read::<TypeId>()?;
        let container = FourCC(subid).is_valid() // the next 8 bytes will need to be a valid header also
                     && FourCC(self.peek::<TypeId>()?).is_valid();
                     //&& self.peek::<Size>(4)? < size - 8

        if container {
            println!("\x1B[A\x1B[{}C-> {}", 14 + depth * 2, FourCC(subid));
            // presume to be a FORM-like list of subchunks
            if let Err(_) = self.parse_subchunks(ESMParser::guesser, size - 4) {
                // rewind the parser on error
                println!("\x1B[A\x1B[{}C       ", 14 + depth * 2);
                print!("\x1B[0G");
                self.seek(pos + *size)?;
            }
        } else {
            // unknown chunk, the only thing left to do is skip
            self.rewind(size - 4)?;
        }

        Ok(*size)
    }
}

type RecordParser<P> = fn(parser: &mut P, header: &RecordHeader) -> chunk_parser::Result<()>;
type FieldParser<P> = fn(parser: &mut P, header: &FieldHeader) -> chunk_parser::Result<()>;

impl<R> ESMParser<R> where R: std::io::Read + std::io::Seek {
    fn read_zstring(&mut self, length: u16) -> chunk_parser::Result<CString> {
        let mut v = Vec::with_capacity(length as usize);
        unsafe {
            let ptr = v.as_mut_ptr();
            self.reader().read_exact(std::slice::from_raw_parts_mut(ptr, length as usize))?;
            v.set_len(length as usize);
        }
        Ok(unsafe { CString::from_vec_unchecked(v) })
    }
    fn read_lstring(&mut self, length: u16) -> chunk_parser::Result<CString> {
        if self.localised { panic!("unimplemented lstring");  }
        else { self.read_zstring(length) }
    }

    pub fn fo3(&mut self, header: &RecordHeader) -> chunk_parser::Result<()> {
        println!("{:?}", header);
        let RecordHeader { size, type_id, flags, .. } = *header;
        match &type_id.0 {
            b"TES4" => {
                if (flags & 0x00000001) != 0 {} // Master (ESM) file
                self.localised = (flags & 0x00000080) != 0;
                if (flags & 0x00000200) != 0 {} // Light Master (ESL) File
                self.parse_fields(|parser, header| {
                    print!("    {:?}", header);
                    match &header.type_id.0 {
                        b"HEDR" => {
                            let HEDR: HEDR = parser.read_fast()?;
                            println!(" {:?}", HEDR);
                        },
                        b"CNAM" => {
                            let CNAM = parser.read_zstring(header.size)?;
                            println!(" {:?}", CNAM);
                        },
                        b"MAST" => {
                            let MAST = parser.read_zstring(header.size)?;
                            println!(" {:?}", MAST);
                        },
                        b"DATA" => {
                            let DATA: u64 = parser.read_fast()?;
                            println!(" {:?}", DATA);
                        },
                        b"ONAM" => {
                            parser.skip(header.size as u32)?;
                            println!(" unimplemented");
                        },
                        _ => { println!(" Unknown typeid '{}'", header.type_id) }
                    }
                    Ok(())
                }, size)?;
            },
            b"GRUP" => {
                let GRUP: GroupHeader = self.read_fast()?;
                println!("    {:?}", GRUP);
                self.skip(size - 24)?;
            },
            b"GLOB" => {
                self.parse_fields(|parser, header| {
                    print!("    {:?}", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!(" {:?}", EDID);
                        },
                        b"FNAM" => {
                            let FNAM: u8 = parser.read_fast()?;
                            println!(" {:?}", FNAM);
                        },
                        b"FLTV" => {
                            let FLTV: f32 = parser.read_fast()?;
                            println!(" {:?}", FLTV);
                        },
                        _ => {
                            parser.skip(header.size as u32)?;
                            println!(" Unknown typeid '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"FACT" => {
                self.parse_fields(|parser, header| {
                    print!("    {:?}", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!(" {:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!(" {:?}", FULL);
                        },
                        b"XNAM" => {
                            let XNAM: XNAM = parser.read_fast()?;
                            println!(" {:?}", XNAM);
                        },
                        b"DATA" => {
                            let DATA: u32 = parser.read_fast()?;
                            println!(" {:#010x}", DATA);
                        },

                        _ => {
                            parser.skip(header.size as u32)?;
                            println!(" Unknown typeid '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"ASPC" => { self.skip(size)?; },
            b"SCPT" => { self.skip(size)?; },
            b"SPEL" => { self.skip(size)?; },
            b"TACT" => { self.skip(size)?; },
            b"ARMO" => { self.skip(size)?; },
            b"DOOR" => { self.skip(size)?; },
            b"MISC" => { self.skip(size)?; },
            b"SCOL" => { self.skip(size)?; },
            b"PWAT" => { self.skip(size)?; },
            b"WEAP" => { self.skip(size)?; },
            b"NPC_" => { self.skip(size)?; },
            b"LVLC" => { self.skip(size)?; },
            b"IDLM" => { self.skip(size)?; },
            b"PROJ" => { self.skip(size)?; },
            b"REGN" => { self.skip(size)?; },
            b"CELL" => { self.skip(size)?; },
            _ => {
                self.skip(size)?;
                println!("Unknown typeid '{}'", type_id);
            }
        }
        Ok(())
    }

    pub fn parse_fields(&mut self, f: FieldParser<Self>, total_size: u32) -> chunk_parser::Result<()> {
        let loop_end = self.reader().stream_position()? + total_size as u64;
        loop {
            let header: FieldHeader = self.read_fast()?;
            let start = self.reader().stream_position()?;
            let size = header.size as u64;
            f(self, &header)?; // the parser function is responsible for parsing the size
            let end = start + size;
            let pos = self.reader().stream_position()?;
            if pos == loop_end { break Ok(()); } // function consumed chunk
            else if pos != end { return Err(chunk_parser::Error::ParseError); } // function made a mistake
        }
    }

    fn parse_records_loop(&mut self, f: RecordParser<Self>, total_size: u64) -> chunk_parser::Result<()> {
        loop {
            let start = self.reader().stream_position()?;
            let header: RecordHeader = self.read_fast()?;
            let size = header.size as u64 + 24;
            f(self, &header)?; // the parser function is responsible for parsing the size
            let end = start + size;
            let pos = self.reader().stream_position()?;
            if pos == total_size { break Ok(()); } // function consumed chunk
            else if pos != end { return Err(chunk_parser::Error::ParseError); } // function made a mistake
        }
    }

    pub fn parse_records(&mut self, f: RecordParser<Self>) -> chunk_parser::Result<()> {
        let total_size = self.reader().seek(std::io::SeekFrom::End(0))?;
        self.reader().seek(std::io::SeekFrom::Start(0))?;
        self.parse_records_loop(f, total_size)
    }
}

//------------------------------------------------------------------------------

pub mod prelude {
    pub use chunk_parser::prelude::*;
    pub use super::ESMParser;
}

//==============================================================================

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    #[allow(non_snake_case)]
    fn Zeta() -> chunk_parser::Result<()> {
        const DATA: &[u8] = include_bytes!("../data/Zeta.esm");
        let mut esm = ESMParser::buf(DATA);
        esm.parse_records(ESMParser::fo3)
    }
}
