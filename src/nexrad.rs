use std::fs::File;
use std::io::Read;
use std::str::from_utf8;

use bincode::{DefaultOptions, Options};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use serde_big_array::BigArray;

use crate::result::{Result, unexpected_error};

pub fn read_nexrad_file(path: &str) -> Result<()> {
    println!("Reading NEXRAD: {:?}", path);

    let mut file = File::open(path)?;

    let file_header: FileHeader = deserialize(&file)?;
    let message_type = file_header.message_type()?
        .ok_or_else(|| unexpected_error("Invalid or unknown message type"))?;

    println!("Date {}, Time {}, Type {:?}",
             file_header.title.file_date,
             file_header.title.file_time,
             message_type);

    match message_type {
        MessageType::Type31 => read_m31(&mut file)?,
    };

    Ok(())
}

fn read_m31(_file: &mut File) -> Result<()> {
    // TODO: parse message format 31
    Ok(())
}

pub fn deserialize<R: Read, S: DeserializeOwned>(t: R) -> Result<S> {
    Ok(DefaultOptions::new()
        .with_fixint_encoding()
        .with_big_endian()
        .deserialize_from(t)?)
}

// Expected maximum is around 366, but this is a safe upper bound.
const MAX_RAYS_IN_SWEEP: usize = 400;

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
struct MessageHeader {
    /// 12 bytes inserted by RPG Communications Mgr. Ignored.
    rpg: [u8; 12],
    /// Message size for this segment, in halfwords
    msg_size: u16,
    /// RDA Redundant Channel
    channel: u8,
    /// Message type. For example, 31
    msg_type: u8,
    /// Msg seq num = 0 to 7FFF, then roll over to 0
    id_seq: u16,
    /// Modified Julian date from 1/1/70
    msg_date: u16,
    /// Packet generation time in ms past midnight
    msg_time: u32,
    /// Number of segments for this message
    num_segs: u16,
    /// Number of this segment
    seg_num: u16,
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
/// A NEXRAD WSR-88D tape header.
struct TapeHeader {
    /// Always ARCHIVE2
    archive2: [u8; 8],

    /// 4-letter site ID. e.g. KLMB
    site_id: [u8; 4],

    /// NCDC tape number. e.g. N00001
    tape_num: [u8; 6],

    /// Blank.
    b1: u8,

    /// Date tape written. dd-MMM-yy e.g. 19-FEB-93
    date: [u8; 9],

    /// Blank.
    b2: u8,

    /// Time tape written. hh:mm:ss. e.g. 10:22:59
    time: [u8; 8],

    /// Blank.
    b3: u8,

    /// Data Center writing tape: RDASC or NCDC.
    data_center: [u8; 5],

    /// WBAN number of this NEXRAD site. This is a unique 5-digit number assigned at NCDC. Numbers
    /// are contained in the NCDC NEXRAD Station History file. The file also contains the four
    /// letter site ID, Latitude, Longitude, Elevation, and common location name.
    wban_num: [u8; 5],

    /// Tape output mode. Current values are 8200, 8500, 8500c.
    tape_mode: [u8; 5],

    /// A volume number to be used for copies and extractions of data from tapes. The form would be
    /// VOL01, VOL02, VOL03 ... VOLnn.
    volume_num: [u8; 5],

    /// Blank. Available for future use.
    b4: [u8; 6],

    /// May be used for internal controls or other information at each archive center. Information
    /// of value to users will be documented at the time of tape shipment.
    #[serde(with = "BigArray")]
    b5: [u8; 31552],
}

#[derive(Debug)]
enum MessageType {
    /// Type for "Build 10", corresponding to an Archive II header filename prefix of "AR2V".
    Type31,
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
struct FileHeader {
    title: Title,
    packet: Packet,
}

impl FileHeader {
    /// Determine this file's expected message type based on the version in the Archive II header's
    /// filename.
    fn message_type(&self) -> Result<Option<MessageType>> {
        let filename = from_utf8(&self.title.filename)?.to_string();

        let message_type = if filename.starts_with("AR2V") {
            Some(MessageType::Type31)
        } else {
            None
        };

        Ok(message_type)
    }
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
/// Title record structure for archive2 data file. The first record of each NEXRAD data file is a
/// title record.
struct Title {
    /// Filename of the archive.
    filename: [u8; 9],

    /// File extension.
    ext: [u8; 3],

    /// Modified Julian date of the file.
    file_date: u32,

    /// Milliseconds of day since midnight of the file.
    file_time: u32,

    /// Unused field.
    unused1: [u8; 4],
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
struct Sweep {
    #[serde(with = "BigArray")]
    rays: [Ray; MAX_RAYS_IN_SWEEP],
}

/// Alias of a packet.
type Ray = Packet;

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
/// Message packet structure
pub struct Packet {
    /// Not used
    ctm: [u8; 12],

    // ======================================
    // Next seven: message header information
    // ======================================

    msg_size: u16,

    /// Digital Radar Data.  This message may contain a combination of either reflectivity, aliased
    /// velocity, or spectrum width.
    msg_type: u16,

    /// I.d. Seq = 0 to 7FFF, then roll over to 0
    id_seq: u16,

    /// Modified Julian date from 1/1/70
    msg_date: u16,

    /// Packet generation time in ms past midnite
    msg_time: u32,

    num_seg: u16,

    seg_num: u16,

    /*
    ===================================
    Next eight: data header information
    ===================================
    */

    /// Collection time for this ray in ms
    ray_time: u32,

    /// Modified Julian date for this ray
    ray_date: u16,

    /// Unambiguous range
    unam_rng: u16,

    /// Coded azimuth angle
    azm: u16,

    /// Ray no. within elevation scan
    ray_num: u16,

    /// Ray status flag
    ray_status: u16,

    /// Coded elevation angle
    elev: u16,

    /// Elevation no. within volume scan
    elev_num: u16,

    // ==============================
    // Next six: gate/bin information
    // ==============================

    /// Range to first gate of refl data
    refl_rng: u16,

    /// Range to first gate of doppler data
    dop_rng: u16,

    /// Refl data gate size
    refl_size: u16,

    /// Doppler data gate size
    dop_size: u16,

    /// No. of reflectivity gates
    num_refl: u16,

    /// No. of doppler gates
    num_dop: u16,

    /// Sector no. within cut
    sec_num: u16,

    /// Gain calibration constant
    sys_cal: f32,

    // ==========================
    // Next five: data parameters
    // ==========================

    /// Reflectivity data ptr
    refl_ptr: u16,

    /// Velocity data ptr
    vel_ptr: u16,

    /// Spectrum width ptr
    spc_ptr: u16,

    /// Doppler velocity resolution
    vel_res: u16,

    /// Volume coverage pattern
    vol_cpat: u16,

    unused1: [u16; 4],

    // ============================================
    // Next three: pointers for Archive II playback
    // ============================================

    ref_ptrp: u16,

    vel_ptrp: u16,

    spc_ptrp: u16,

    /// Nyquist velocity
    nyq_vel: u16,

    /// Atmospheric attenuation factor
    atm_att: u16,

    min_dif: u16,

    unused2: [u16; 17],

    #[serde(with = "BigArray")]
    data: [u8; 2300],

    /// Frame check sequence
    fts: [u8; 4],
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
/// A radar site.
struct RadarSite {
    /// Arbitrary number of this radar site.
    number: u32,

    /// Nexrad site name.
    name: [char; 4],

    /// Nearest city to radar site.
    city: [char; 15],

    /// State of radar site.
    state: [char; 2],

    /// Degrees of latitude of site.
    latd: u32,

    /// Minutes of latitude of site.
    latm: u32,

    /// Seconds of latitude of site.
    lats: u32,

    /// Degrees of longitude of site.
    lond: u32,

    /// Minutes of longitude of site.
    lonm: u32,

    /// Seconds of longitude of site.
    lons: u32,

    /// Height of site in meters above sea level.
    height: u32,

    /// Bandwidth of site (MHz).
    bwidth: u32,

    /// Length of short pulse (ns).
    spulse: u32,

    /// Length of long pulse (ns).
    lpulse: u32,
}