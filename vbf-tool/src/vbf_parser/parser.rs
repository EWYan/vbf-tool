extern crate json;
extern crate sha2;

use sha2::{Digest, Sha256};
use std::io::prelude::*;
use std::{fs, io};
//CRC_16_IBM_3740 -> CCITT,AUTOSAR
//need self defined CRC32 
use crc::{Algorithm, Crc, CRC_16_IBM_3740};
use std::mem::transmute;
use std::time::Instant;
pub const CRC_32_IEEE802: Algorithm<u32> = Algorithm {
    poly: 0x04c11db7,
    init: 0xffffffff,
    refin: true,
    refout: true,
    xorout: 0xffffffff,
    check: 0xcbf43926,
    residue: 0xdebb20e3,
};

pub struct VbfFt {
    status: StatusT,
    vbt_info: VbtInfo,
    script: ScriptsT,
}
// todo add erase field
impl VbfFt {
    pub fn new(vbb_path: &str) -> io::Result<()> {
        let vbb_json = fs::read_to_string(vbb_path).unwrap();
        let vbb_parsed = json::parse(&vbb_json).unwrap();
        let mut vbf_inst = VbfFt {
            status: StatusT::Init,
            vbt_info: VbtInfo {
                vbt_len: 44_u32,
                vbt_hash: [0_u8; 32],
                vbt_format: 0_u16,
                num_blk: 0_u16,
                blk: [
                    blk_info {
                        start_addr: 0x01000000 as u32,
                        length: 0_u32,
                        hash_value: [0_u8; 32],
                    },
                    blk_info {
                        start_addr: 0_u32,
                        length: 0_u32,
                        hash_value: [0_u8; 32],
                    },
                ],
            },
            script: ScriptsT {
                source_file: Item {
                    description: String::from("blahblah"),
                    value: ValueT::Literal(String::from(
                        vbb_parsed["VBF1"]["SourceFile"].as_str().unwrap(),
                    )),
                },
                target_file: Item {
                    description: String::from("blahblah"),
                    value: ValueT::Literal(String::from(
                        vbb_parsed["VBF1"]["TargetFile"].as_str().unwrap(),
                    )),
                },
                vbf_version: Item {
                    description: String::from("blahblah"),
                    value: ValueT::Literal(String::from(
                        vbb_parsed["VBF1"]["VBFVersion"].as_str().unwrap(),
                    )),
                },
                sw_type: Item {
                    description: String::from("software part type: Executable"),
                    value: ValueT::Literal(String::from(
                        vbb_parsed["VBF1"]["SwType"].as_str().unwrap(),
                    )),
                },
                sw_part_nmu: Item {
                    description: String::from("software part number:"),
                    value: ValueT::Literal(String::from(
                        vbb_parsed["VBF1"]["SwPartNum"].as_str().unwrap(),
                    )),
                },
                ecu_addr: Item {
                    description: String::from("ECU Address:"),
                    value: ValueT::Literal(String::from(
                        vbb_parsed["VBF1"]["ECUaddr"].as_str().unwrap(),
                    )),
                },
                sw_version: Item {
                    description: String::from("Software Version:"),
                    value: ValueT::Literal(String::from(
                        vbb_parsed["VBF1"]["SwVersion"].as_str().unwrap(),
                    )),
                },
                image_offset: Item { 
                    description: String::from("image offset"),
                    value: ValueT::Literal(String::from(
                        vbb_parsed["VBF1"]["ImageOffset"]
                            .as_str()
                            .unwrap(),
                    )),
                 },
                create_vbt: Item {
                    description: String::from("If enable VBT"),
                    value: ValueT::Toggle(
                        vbb_parsed["VBF1"]["CreateVerificationBlock"]
                            .as_bool()
                            .unwrap(),
                    ),
                },
                vbt_addr: Item {
                    description: String::from("Start address of the VBT"),
                    value: ValueT::Literal(String::from(
                        vbb_parsed["VBF1"]["VerificationBlockStartAddr"]
                            .as_str()
                            .unwrap(),
                    )),
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
        let bin_file_path = vbf_inst.script.source_file.value.literal().unwrap();
        let mut bin_file = fs::File::open(bin_file_path).unwrap();
        let mut hasher = Sha256::new();
        let n = io::copy(&mut bin_file, &mut hasher).unwrap();
        let hash = hasher.finalize();
        vbf_inst.vbt_info.blk[0].length = n as u32;
        vbf_inst.vbt_info.vbt_format = 0_u16;
        vbf_inst.vbt_info.num_blk = 1_u16;
        vbf_inst.vbt_info.blk[0]
            .hash_value
            .clone_from_slice(hash.as_slice());

        // try to create new vbf files
        let target_file_path = vbf_inst.script.target_file.value.literal().unwrap();
        let mut file_w = fs::File::create(target_file_path).unwrap();

        // write trivial option parameters
        VbfFt::dump(&mut vbf_inst, &mut file_w).unwrap();

        // write all metadata to disk
        file_w.sync_all().unwrap();
        Ok(())
    }
    // dump data to vbf files
    pub fn dump(&mut self, fp: &mut fs::File) -> io::Result<()> {
        // vbf_version =
        fp.write_fmt(format_args!(
            "vbf_version = {};\r\n",
            self.script.vbf_version.value.literal().unwrap()
        ))
        .unwrap();
        // Freetech
        fp.write(b"\r\n").unwrap();
        fp.write(b"header {\r\n").unwrap();
        fp.write(b"    //**********************************************************\r\n")
            .unwrap();
        fp.write(b"    //*\r\n").unwrap();
        fp.write(b"    //*                      Freetech.co\r\n")
            .unwrap();
        fp.write(b"    //*\r\n").unwrap();
        fp.write(b"    //*     This file is generated by VBF CONVERT ver. 5.10.0\r\n")
            .unwrap();
        fp.write(b"    //*\r\n").unwrap();
        fp.write(b"    //*                        DO NOT EDIT !\r\n")
            .unwrap();
        fp.write(b"    //*\r\n").unwrap();
        fp.write(b"    //**********************************************************\r\n")
            .unwrap();
        fp.write(b"    \r\n").unwrap();

        fp.write(b"    // Volvo software part number\r\n").unwrap();
        fp.write_fmt(format_args!(
            "       sw_part_number = \"{}\";\r\n",
            self.script.sw_part_nmu.value.literal().unwrap()
        ))
        .unwrap();
        fp.write(b"\r\n").unwrap();

        fp.write(b"    // Software Version: \r\n").unwrap();
        fp.write_fmt(format_args!(
            "       sw_version = \"{}\";\r\n",
            self.script.sw_version.value.literal().unwrap()
        ))
        .unwrap();
        fp.write(b"\r\n").unwrap();

        fp.write(b"    // Volvo software part type: Executable\r\n")
            .unwrap();
        fp.write_fmt(format_args!(
            "       sw_part_type = {};\r\n",
            self.script.sw_type.value.literal().unwrap()
        ))
        .unwrap();
        fp.write(b"\r\n").unwrap();

        fp.write(b"    // Data format identifier: 0x00 = Uncompressed, 0x10 = Compressed\r\n")
            .unwrap();
        fp.write_fmt(format_args!(
            "       data_format_identifier = {};\r\n",
            match self.script.compressed.value.toggle().unwrap() {
                true => "0x10",
                false => "0x00",
            }
        ))
        .unwrap();
        fp.write(b"\r\n").unwrap();

        fp.write(b"    // ECU Address: \r\n").unwrap();
        fp.write_fmt(format_args!(
            "       ecu_address = 0x{};\r\n",
            self.script.ecu_addr.value.literal().unwrap()
        ))
        .unwrap();
        fp.write(b"\r\n").unwrap();

        // vbt info
        let vbt_enable = self.script.create_vbt.value.toggle().unwrap();
        if vbt_enable {
            self.vbt_info.num_blk += 1_u16;
            fp.write(b"    // Start address of the hash table\r\n")
                .unwrap();
            fp.write_fmt(format_args!(
                "       verification_block_start = 0x{:08X};\r\n",
                self.script.vbt_addr.value.literal().unwrap().parse::<u32>().unwrap()
            ))
            .unwrap();
            fp.write(b"\r\n").unwrap();

            fp.write(b"    // Length of the hash table\r\n").unwrap();
            fp.write_fmt(format_args!(
                "       verification_block_length = 0x{:08X};\r\n",
                self.vbt_info.vbt_len
            ))
            .unwrap();
            fp.write(b"\r\n").unwrap();

            // calculate hash digest of vbt
            let mut hasher = Sha256::new();
            let vbt_address: [u8; 4] = unsafe {
                transmute(
                    self.script
                        .vbt_addr
                        .value
                        .literal()
                        .unwrap()
                        .parse::<u32>()
                        .unwrap()
                        .to_be(),
                )
            };
            hasher.update(vbt_address);
            let vbt_length: [u8; 4] = unsafe { transmute(self.vbt_info.vbt_len.to_be()) };
            hasher.update(vbt_length);
            let vbt_format: [u8; 2] = unsafe { transmute(self.vbt_info.vbt_format.to_be()) };
            hasher.update(vbt_format);
            let num_blk: [u8; 2] = unsafe { transmute((self.vbt_info.num_blk - 1).to_be()) };
            hasher.update(num_blk);

            let bytes: [u8; 4] = unsafe { transmute(self.vbt_info.blk[0].start_addr.to_be()) };
            hasher.update(&bytes);
            let bytes: [u8; 4] = unsafe { transmute(self.vbt_info.blk[0].length.to_be()) };
            hasher.update(&bytes);

            hasher.update(self.vbt_info.blk[0].hash_value);
            let vbt_hash = hasher.finalize();
            // store final vbt_hash
            self.vbt_info.vbt_hash.clone_from_slice(vbt_hash.as_slice());

            fp.write(b"    // Root hash value\r\n").unwrap();
            fp.write(b"       verification_block_root_hash = 0x")
                .unwrap();
            // fp.write(&self.vbt_info.vbt_hash[..]).unwrap();
            for b in self.vbt_info.vbt_hash {
                fp.write_fmt(format_args!("{:02X}", b)).unwrap();
            }

            fp.write(b";\r\n\r\n").unwrap();
        }

        fp.write(b"    // Blocks sorted\r\n").unwrap();
        fp.write(b"    // Blocks grouped\r\n").unwrap();
        fp.write_fmt(format_args!(
            "    // Blocks:   {};\r\n",
            1 + (vbt_enable as u32)
        ))
        .unwrap();
        fp.write_fmt(format_args!(
            "    // Bytes:    {};\r\n",
            self.vbt_info.blk[0].length + 44 * (vbt_enable as u32)
        ))
        .unwrap();

        fp.write(b"\r\n").unwrap();

        // write crc
        let crc32_inst = Crc::<u32>::new(&CRC_32_IEEE802);
        let crc16_inst = Crc::<u16>::new(&CRC_16_IBM_3740);
        let mut bin_crc32 = crc32_inst.digest();
        let mut bin_crc16 = crc16_inst.digest();
        
        let mut bin_size: u32 = 0;
        let mut f_bin = fs::File::open(self.script.source_file.value.literal().unwrap()).unwrap();
        let crc_v = loop {
            let mut buffer = [0; 4096];
            let n = f_bin.read(&mut buffer[..]).unwrap() as usize;
            bin_crc32.update(&buffer[..n]);
            bin_crc16.update(&buffer[..n]);
            bin_size += n as u32;
            if n < 4096 {
                break (bin_crc32.finalize(), bin_crc16.finalize());
            }
        };
        // file_checksum
        fp.write_fmt(format_args!("    file_checksum = 0x{:08X};\r\n", crc_v.0))
            .unwrap();
        fp.write_all(b"}").unwrap();

        // seek bin file to start
        f_bin.seek(io::SeekFrom::Start(0)).unwrap();

        let bytes: [u8; 4] = unsafe { transmute(self.vbt_info.blk[0].start_addr.to_be()) };
        fp.write_all(&bytes).unwrap();
        let bytes: [u8; 4] = unsafe { transmute(self.vbt_info.blk[0].length.to_be()) };
        fp.write_all(&bytes).unwrap();

        // write binary of bin to target file
        let mut write_cnt = 0_usize;
        loop {
            let mut buffer = [0; 4096];
            let n = f_bin.read(&mut buffer[..]).unwrap();
            write_cnt += n;
            match n {
                4096 => {
                    fp.write(&buffer[..n]).unwrap();
                }
                0 => {
                    break;
                }
                w => {
                    fp.write(&buffer[..w]).unwrap();
                }
            }
        }
        let bytes: [u8; 2] = unsafe { transmute(crc_v.1.to_be()) };
        fp.write(&bytes[..2]).unwrap();
        if vbt_enable {
            let vbt_address: [u8; 4] = unsafe {
                transmute(
                    self.script
                        .vbt_addr
                        .value
                        .literal()
                        .unwrap()
                        .parse::<u32>()
                        .unwrap()
                        .to_be(),
                )
            };
            fp.write(&vbt_address).unwrap();
            let vbt_length: [u8; 4] = unsafe { transmute(self.vbt_info.vbt_len.to_be()) };
            fp.write(&vbt_length).unwrap();

            let crc16_inst = Crc::<u16>::new(&CRC_16_IBM_3740);
            let mut crc16_vbt = crc16_inst.digest();
            let vbt_format: [u8; 2] = unsafe { transmute(self.vbt_info.vbt_format.to_be()) };
            crc16_vbt.update(&vbt_format);
            fp.write(&vbt_format).unwrap();

            let num_blk: [u8; 2] = unsafe { transmute((self.vbt_info.num_blk -1).to_be()) };
            crc16_vbt.update(&num_blk);
            fp.write(&num_blk).unwrap();

            let bytes: [u8; 4] = unsafe { transmute(self.vbt_info.blk[0].start_addr.to_be()) };
            crc16_vbt.update(&bytes);
            fp.write_all(&bytes).unwrap();
            let bytes: [u8; 4] = unsafe { transmute(self.vbt_info.blk[0].length.to_be()) };
            crc16_vbt.update(&bytes);
            fp.write_all(&bytes).unwrap();

            crc16_vbt.update(&self.vbt_info.blk[0].hash_value);
            let vbt_check: [u8; 2] = unsafe { transmute(crc16_vbt.finalize().to_be()) };

            fp.write(&self.vbt_info.blk[0].hash_value).unwrap();

            fp.write(&vbt_check).unwrap();
        }
        #[cfg(debug_assertions)]
        {
            println!("write_cnt: {}", write_cnt);
            println!("crc16:0x{:04X}", crc_v.1);
            println!("crc32:0x{:08X}", crc_v.0);
            println!("bin_size:0x{:0X}", bin_size);
        }
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
    source_file: Item, //input bin file
    target_file: Item, //output vbf file
    vbf_version: Item, //vbf format version
    sw_type: Item,     //software type,
    sw_part_nmu: Item, //software partnumber
    ecu_addr: Item,    //ecu address
    sw_version: Item,  //software version
    image_offset: Item,//image offset
    create_vbt: Item,  //if create verification block table
    vbt_addr: Item,    //address of vbt
    compressed: Item,  //if compress input bin file
    sort: Item,        //tbd
    group: Item,       //tbd
}
#[derive(Debug)]
struct Item {
    description: String,
    value: ValueT,
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

struct blk_info {
    start_addr: u32,
    length: u32,
    hash_value: [u8; 32],
}
struct VbtInfo {
    vbt_len: u32,
    vbt_hash: [u8; 32],
    vbt_format: u16,
    num_blk: u16,
    blk: [blk_info; 2],
}
