extern crate json;
extern crate sha2;

use std::{fs, io};
use std::io::prelude::*;
use sha2::{Sha256, Digest};
//CRC_16_IBM_3740 -> CCITT,AUTOSAR
use crc::{Crc, Algorithm, CRC_16_IBM_3740};
use std::mem::transmute;
use std::time::Instant;
pub const CRC_32_IEEE802: Algorithm<u32> = Algorithm { poly: 0x04c11db7, init: 0xffffffff, refin: true, refout: true, xorout: 0xffffffff, check: 0xcbf43926, residue: 0xdebb20e3 };

pub struct VbfFt{
    status:StatusT,
    file_bytes:u32,
    script:ScriptsT,
}

impl VbfFt {
    pub fn new(vbb_path: &str) -> io::Result<()> {
        let vbb_json = fs::read_to_string(vbb_path).unwrap();
        let vbb_parsed = json::parse(&vbb_json).unwrap();
        let mut vbf_inst = VbfFt{
            status: StatusT::Init,
            file_bytes: 0u32,
            script: ScriptsT{
                source_file: Item {
                    description: String::from("blahblah"),
                    value: ValueT::Literal(String::from(vbb_parsed["VBF1"]["SourceFile"].as_str().unwrap())),
                },
                target_file: Item { 
                    description: String::from("blahblah"), 
                    value: ValueT::Literal(String::from(vbb_parsed["VBF1"]["TargetFile"].as_str().unwrap())), 
                },
                vbf_version: Item { 
                    description: String::from("blahblah"), 
                    value: ValueT::Literal( String::from(vbb_parsed["VBF1"]["VBFVersion"].as_str().unwrap())), 
                },
                sw_type: Item { 
                    description: String::from("software part type: Executable"), 
                    value: ValueT::Literal( String::from(vbb_parsed["VBF1"]["SwType"].as_str().unwrap())), 
                },
                sw_part_nmu: Item { 
                    description: String::from("software part number:"), 
                    value: ValueT::Literal( String::from(vbb_parsed["VBF1"]["SwPartNum"].as_str().unwrap())), 
                },
                ecu_addr: Item { 
                    description: String::from("ECU Address:"), 
                    value: ValueT::Literal( String::from(vbb_parsed["VBF1"]["ECUaddr"].as_str().unwrap())), 
                },
                sw_version: Item { 
                    description: String::from("Software Version:"), 
                    value: ValueT::Literal( String::from(vbb_parsed["VBF1"]["SwVersion"].as_str().unwrap())), 
                },
                create_vbt: Item { 
                    description: String::from("If enable VBT"), 
                    value: ValueT::Toggle(vbb_parsed["VBF1"]["CreateVerificationBlock"].as_bool().unwrap()), 
                },
                vbt_addr: Item { 
                    description: String::from("Start address of the VBT"), 
                    value: ValueT::Literal( String::from(vbb_parsed["VBF1"]["VerificationBlockStartAddr"].as_str().unwrap())), 
                },
                compressed: Item { 
                    description: String::from("If enable compress fea:"), 
                    value: ValueT::Toggle(vbb_parsed["VBF1"]["Compressed"].as_bool().unwrap()), 
                },
                sort: Item { 
                    description: String::from("Sorted"), 
                    value: ValueT::Toggle(vbb_parsed["VBF1"]["Sort"].as_bool().unwrap()), 
                },
                group: Item { 
                    description: String::from("Grouped"), 
                    value: ValueT::Toggle(vbb_parsed["VBF1"]["Group"].as_bool().unwrap()), 
                },
            },
        };
        // calculate bin file hash256
        let now = Instant::now();
        let bin_file_path = vbf_inst.script.source_file.value.literal().unwrap();
        let mut bin_file = fs::File::open(bin_file_path).unwrap();
        let mut hasher = Sha256::new();
        let n = io::copy(&mut bin_file, &mut hasher).unwrap();
        let hash = hasher.finalize();
        println!("binary hash({:x?}) :{:x?}", n,hash);
        println!("elapsed time :{:?}",now.elapsed());
        vbf_inst.file_bytes = n as u32;
        // try to create new vbf files
        let target_file_path = vbf_inst.script.target_file.value.literal().unwrap();
        let mut file_w = fs::File::create(target_file_path).unwrap();

        // write trivial option parameters
        VbfFt::dump(&vbf_inst, &mut file_w).unwrap();

        // write all metadata to disk
        file_w.sync_all().unwrap();
        Ok(())
    }
    // dump data to vbf files
    pub fn dump(&self, fp: &mut fs::File) -> io::Result<()>{
        // vbf_version = 
        fp.write_fmt(format_args!("vbf_version = {};\r\n", self.script.vbf_version.value.literal().unwrap())).unwrap();
        // fp.write(self.script.vbf_version.value.literal().unwrap().as_bytes()).unwrap();

        fp.write(b"\r\n").unwrap();
        fp.write(b"header {\r\n").unwrap();
        fp.write(b"    //**********************************************************\r\n").unwrap();
        fp.write(b"    //*\r\n").unwrap();
        fp.write(b"    //*                  Volvo Car Corporation\r\n").unwrap();
        fp.write(b"    //*\r\n").unwrap();
        fp.write(b"    //*     This file is generated by VBF CONVERT ver. 5.10.0\r\n").unwrap();
        fp.write(b"    //*\r\n").unwrap();
        fp.write(b"    //*                        DO NOT EDIT !\r\n").unwrap();
        fp.write(b"    //*\r\n").unwrap();
        fp.write(b"    //**********************************************************\r\n").unwrap();
        fp.write(b"    \r\n").unwrap();

        fp.write(b"    // Volvo software part number\r\n").unwrap();
        fp.write_fmt(format_args!("       sw_part_number = \"{}\";\r\n", self.script.sw_part_nmu.value.literal().unwrap())).unwrap();
        fp.write(b"\r\n").unwrap();

        fp.write(b"    // Software Version: \r\n").unwrap();
        fp.write_fmt(format_args!("       sw_version = \"{}\";\r\n", self.script.sw_version.value.literal().unwrap())).unwrap();
        fp.write(b"\r\n").unwrap();

        fp.write(b"    // Volvo software part type: Executable\r\n").unwrap();
        fp.write_fmt(format_args!("       sw_part_type = {};\r\n", self.script.sw_type.value.literal().unwrap())).unwrap();
        fp.write(b"\r\n").unwrap();
        
        fp.write(b"    // Data format identifier: 0x00 = Uncompressed, 0x10 = Compressed\r\n").unwrap();
        fp.write_fmt(format_args!("       data_format_identifier = {};\r\n", match self.script.compressed.value.toggle().unwrap() {
                                                                                   true => "0x10",
                                                                                   false => "0x00"
                                                                                    })).unwrap();
        fp.write(b"\r\n").unwrap();
        
        fp.write(b"    // ECU Address: \r\n").unwrap();
        fp.write_fmt(format_args!("       ecu_address = 0x{};\r\n", self.script.ecu_addr.value.literal().unwrap())).unwrap();
        fp.write(b"\r\n").unwrap();
        
        fp.write(b"    // Blocks sorted\r\n").unwrap();
        fp.write(b"    // Blocks grouped\r\n").unwrap();
        fp.write(b"    // Blocks:   1\r\n").unwrap();
        fp.write_fmt(format_args!("    // Bytes:    {};\r\n", self.file_bytes)).unwrap();

        fp.write(b"\r\n").unwrap();

        // write crc
        let now = Instant::now();
        let crc32_inst = Crc::<u32>::new(&CRC_32_IEEE802);
        let crc16_inst = Crc::<u16>::new(&CRC_16_IBM_3740);
        let mut bin_crc32 = crc32_inst.digest();
        let mut bin_crc16 = crc16_inst.digest();
        let mut bin_size:u32 = 0;
        // let path2 = Path::new("case3-calcrc");
        let mut f_bin = fs::File::open(self.script.source_file.value.literal().unwrap()).unwrap();
        // let mut file_size:usize = 0;
        let crc_v = loop {
            let mut buffer = [0;4096];
            let n = f_bin.read(&mut buffer[..]).unwrap() as usize;
            bin_crc32.update(&buffer[..n]);
            bin_crc16.update(&buffer[..n]);
            bin_size += n as u32;
            if n < 4096 {
                break (bin_crc32.finalize(), bin_crc16.finalize())
            }
        };
        println!("elapsed time: {:?}",now.elapsed());
        // file_checksum
        fp.write_fmt(format_args!("    file_checksum = 0x{:08X};\r\n", crc_v.0)).unwrap();
        fp.write_all(b"}").unwrap();
        
        // seek bin file to start 
        f_bin.seek(io::SeekFrom::Start(0)).unwrap();
        let start_address = 00000000_u32;
        let bytes: [u8; 4] = unsafe { transmute(start_address.to_be()) };
        fp.write_all(&bytes).unwrap();
        let bytes: [u8; 4] = unsafe { transmute(bin_size.to_be()) };
        fp.write_all(&bytes).unwrap();

        // write binary of bin to target file
        let now = Instant::now();
        let mut write_cnt = 0_usize;
        loop {
            let mut buffer = [0;4096];
            let n = f_bin.read(&mut buffer[..]).unwrap();
            println!("current read size:{}", n);
            write_cnt += n;
            if n < 4096 {
                fp.write(&buffer[..n]).unwrap();
                let bytes: [u8; 2] = unsafe { transmute(crc_v.1.to_be()) };
                fp.write(&bytes[..2]).unwrap();
                break;
            } else {
                fp.write(&buffer[..n]).unwrap();
            }
            if n == 4096 {
                let bytes: [u8; 2] = unsafe { transmute(crc_v.1.to_be()) };
                fp.write(&bytes[..2]).unwrap();
            }
        };
        println!("write consumed time:{:?}", now.elapsed());
        println!("write_cnt: {:x}", write_cnt);
        println!("crc16:0x{:04X}", crc_v.1);
        println!("crc32:0x{:08X}", crc_v.0);
        println!("bin_size:0x{:0X}", bin_size);

        #[cfg(debug_assertions)]
        println!("test debug");
        #[cfg(not(debug_assertions))]
        println!("test release");
        Ok(())
    }
}
#[derive(Debug)]
enum StatusT {
    Init,
    Uninit,
}
#[derive(Debug)]
struct ScriptsT {
    source_file: Item,//input bin file
    target_file: Item,//output vbf file
    vbf_version: Item,//vbf format version 
    sw_type: Item, //software type,
    sw_part_nmu: Item,//software partnumber
    ecu_addr: Item,//ecu address
    sw_version: Item,//software version
    create_vbt: Item,//if create verification block table
    vbt_addr: Item,//address of vbt
    compressed: Item,//if compress input bin file
    sort: Item,//tbd
    group: Item,//tbd
}
#[derive(Debug)]
struct Item {
    description: String,
    value:ValueT,
}
#[derive(Debug)]
enum ValueT {
    Literal(String),
    Toggle(bool),
}
impl ValueT {
    fn literal(&self) -> Option<String> {
        match self {
            ValueT::Literal(c) => Some(c.to_string()),
            _ => None,
        }
    }    
    fn toggle(&self) -> Option<bool> {
        match self {
            ValueT::Toggle(c) => Some(*c),
            _ => None,
        }
    }    
}

// fn get_crc()