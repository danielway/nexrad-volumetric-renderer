use std::fs::File;
use std::io::{Seek, SeekFrom};

use crate::result::Result;
use crate::utility::{read_int, read_string};

pub fn read_volume_scan(path: &str) -> Result<()> {
    let mut file = File::open(path).expect("can open file");

    let data_format = read_string(&mut file, 8)?;

    file.seek(SeekFrom::Current(4))?;

    let julian_day = read_int(&mut file)?;
    let milliseconds = read_int(&mut file)?;

    println!("{} {} {}", data_format, julian_day, milliseconds);

    file.seek(SeekFrom::Current(4))?; // station, only relevant in AR2V0001

    if data_format.starts_with("AR2V") {
        file.seek(SeekFrom::Current(4))?;

        let bz = read_string(&mut file, 2)?;
        if bz == "BZ" {
            // todo: uncompress
        }
    }

    Ok(())
}