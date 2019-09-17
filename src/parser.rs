use std::{
    fs,
    path::Path,
};
use serde_xml_rs::from_reader;


#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Read {
    #[serde(rename = "Number", default)]
    pub number : u64, // parsed from run info, outputed in fastq
    #[serde(rename = "NumCycles", default)]
    pub num_cycles : u64, // number of cycles expected for one read (~100)
    #[serde(rename = "IsIndexedRead", default)]
    pub is_indexed_read : String,
}


#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct FlowcellLayout {
    #[serde(rename = "LaneCount", default)]
    pub lane_count : u32,
    #[serde(rename = "SurfaceCount", default)]
    pub surface_count : u32,
    #[serde(rename = "SwathCount", default)]
    pub swath_count : u64,
    #[serde(rename = "TileCount", default)]
    pub tile_count : u64,
    #[serde(rename = "FlowcellSide", default)]
    pub flowcell_side : u32,
    #[serde(rename = "TileSet", default)]
    pub tile_set : Vec<TileSet>,
}


#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct TileSet {
    #[serde(rename = "TileNamingConvention", default)]
    pub tile_naming_convention : String,
    #[serde(rename = "Tiles", default)]
    pub tiles : Vec<Tiles>
}


#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Tiles {
    #[serde(rename = "Tile", default)]
    pub tile : Vec<String>,
}


#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Reads {
    #[serde(rename = "Read", default)]
    pub read : Vec<Read>,
}


#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Run {
    #[serde(rename = "Id", default)]
    pub id : String, // parsed from run info, outputed in fastq
    #[serde(rename = "Number", default)]
    pub number : u64,
    #[serde(rename = "Flowcell", default)]
    pub flowcell : String,
    #[serde(rename = "Instrument", default)]
    pub instrument : String,
    #[serde(rename = "Date", default)]
    pub date : String,
    #[serde(rename = "Reads", default)]
    pub reads : Vec<Reads>,
    #[serde(rename = "FlowcellLayout", default)]
    pub flow_cell_layout : Vec<FlowcellLayout>,
}


#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct RunInfo {
    #[serde(rename = "Version", default)]
    pub version : u32,
    #[serde(rename = "Run", default)]
    pub runs : Vec<Run>
}


pub fn parse_run_info(run_info_path: &Path) -> RunInfo {
    println!("reading file {}", run_info_path.display());
    let run_xml = fs::read_to_string(run_info_path).expect("error reading the file");
    let runinfo : RunInfo = from_reader(run_xml.as_bytes()).unwrap();
    println!("{:#?}", runinfo);
    return runinfo
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_runinfo() {
        let filename_info = Path::new("test_data/190414_A00111_0296_AHJCWWDSXX/RunInfo.xml");
        let actual_runinfo : RunInfo = parse_run_info(filename_info);
        let expected_runinfo =
            RunInfo {
                version: 5,
                runs: vec![
                    Run {
                        id: "190414_A00111_0296_AHJCWWDSXX".to_string(),
                        number: 296,
                        flowcell: "HJCWWDSXX".to_string(),
                        instrument: "A00111".to_string(),
                        date: "4/14/2019 1:17:20 PM".to_string(),
                        reads: vec![
                            Reads {
                                read: vec![
                                    Read {
                                        number: 1,
                                        num_cycles: 4,
                                        is_indexed_read: "N".to_string()
                                    },
                                    Read {
                                        number: 2,
                                        num_cycles: 8,
                                        is_indexed_read: "Y".to_string()
                                    },
                                    Read {
                                        number: 3,
                                        num_cycles: 8,
                                        is_indexed_read: "Y".to_string()
                                    },
                                    Read {
                                        number: 4,
                                        num_cycles: 4,
                                        is_indexed_read: "N".to_string()
                                    }
                                ]
                            }
                        ],
                        flow_cell_layout: vec![
                            FlowcellLayout {
                                lane_count: 1,
                                surface_count: 1,
                                swath_count: 6,
                                tile_count: 3,
                                flowcell_side: 1,
                                tile_set: vec![
                                    TileSet {
                                        tile_naming_convention: "FourDigit".to_string(),
                                        tiles: vec![
                                            Tiles {
                                                tile: vec![
                                                    "1_1101".to_string(),
                                                    "1_1102".to_string(),
                                                    "1_1103".to_string()
                                                ]
                                            }
                                        ]
                                    }
                                ]
                            }
                        ]
                    }
                ]
            };
        assert_eq!(actual_runinfo, expected_runinfo)

    }
}
