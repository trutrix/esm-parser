//! Elder Scrolls Mod file parser.

use chunk_parser::prelude::*;
use esm_bindings::*;

//------------------------------------------------------------------------------

#[chunk_parser]
pub struct ESMParser;

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

impl<R> ESMParser<R> where R: std::io::Read + std::io::Seek {
    #[allow(non_snake_case)]
    pub fn TES4(&mut self, ( typeid, size ): &<ESMParser<R> as Parser>::Header) -> chunk_parser::Result<u32> {
        match typeid {
            b"TES4" => {
                let header: RecordHeader = self.read_fast()?;
                println!("{:?}", header);
                self.skip(*size - 16)?;
            },
            b"GRUP" => {
                let header: GroupHeader = self.read_fast()?;
                println!("{:?}", header);
                self.skip(*size - 16)?;
            },
            b"GLOB" => { self.skip(*size)?; },
            b"FACT" => { self.skip(*size)?; },
            b"ASPC" => { self.skip(*size)?; },
            b"SCPT" => { self.skip(*size)?; },
            b"SPEL" => { self.skip(*size)?; },
            b"TACT" => { self.skip(*size)?; },
            b"ARMO" => { self.skip(*size)?; },
            b"DOOR" => { self.skip(*size)?; },
            b"MISC" => { self.skip(*size)?; },
            b"SCOL" => { self.skip(*size)?; },
            b"PWAT" => { self.skip(*size)?; },
            b"WEAP" => { self.skip(*size)?; },
            b"NPC_" => { self.skip(*size)?; },
            b"LVLC" => { self.skip(*size)?; },
            b"IDLM" => { self.skip(*size)?; },
            b"PROJ" => { self.skip(*size)?; },
            b"REGN" => { self.skip(*size)?; },
            b"CELL" => { self.skip(*size)?; },
            //b"EDID" => { self.skip(*size)?; },
            _ => { self.guesser(&( *typeid, *size ))?; }
        }
        Ok(*size)
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
    fn Zeta() {
        const DATA: &[u8] = include_bytes!("../data/Zeta.esm");
        let mut esm = ESMParser::buf(DATA);
        esm.parse(ESMParser::TES4).unwrap();
    }
}
