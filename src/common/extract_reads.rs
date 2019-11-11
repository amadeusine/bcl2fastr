//! Extract and decompress a set of tiles from a vector of cbcl files.

use std::{
    fs::File,
    io::prelude::*,
    io::SeekFrom,
};

use flate2::read::MultiGzDecoder;
use ndarray::{Array3, ArrayView, ArrayViewMut2, Axis};

use crate::cbcl_header_decoder::CBCLHeader;


/// converts from 0..3 values to the appropriate base, or N if the qscore is too low
fn u8_to_base(b: u8, q: u8) -> u8 {
    if q <= 35 { return b'N' }

    match b {
        0 => b'A',
        1 => b'C',
        2 => b'G',
        3 => b'T',
        _ => b'N',
    }
}


/// unpacks a single byte into four 2-bit integers
fn unpack_byte(b: &u8, filter: &[bool], header: &CBCLHeader) -> Vec<u8> {
    let q_1 = header.decode_qscore((b >> 6) & 3u8);
    let b_1 = u8_to_base((b >> 4) & 3u8, q_1);
    let q_2 = header.decode_qscore((b >> 2) & 3u8);
    let b_2 = u8_to_base(b & 3u8, q_2);

    match filter {
        [true, true] => vec![b_2, q_2, b_1, q_1],
        [true, false] =>  vec![b_2, q_2],
        [false, true] =>  vec![b_1, q_1],
        _ => vec![],
    }
}


/// extract multiple tiles from a CBCL file and return decompressed bytes
fn extract_tiles(header: &CBCLHeader, i: usize) -> std::io::Result<Vec<u8>> {
    let start_pos = header.start_pos[i];
    let uncompressed_size = header.uncompressed_size[i];
    let compressed_size = header.compressed_size[i];

    // open file and seek to start position
    let mut cbcl = File::open(&header.cbcl_path)?;
    cbcl.seek(SeekFrom::Start(start_pos))?;

    // read the compressed data for specified tile(s)
    let mut read_buffer = vec![0u8; compressed_size];
    cbcl.read_exact(&mut read_buffer)?;

    // use MultiGzDecoder to uncompress the number of bytes summed 
    // over the offsets of all tile_idces
    let mut uncomp_bytes = vec![0u8; uncompressed_size];
    let mut gz = MultiGzDecoder::new(&read_buffer[..]);
    gz.read_exact(&mut uncomp_bytes)?;

    Ok(uncomp_bytes)
}


/// given a CBCL file and some tiles: extract, translate and filter the bases+scores
fn process_tiles(
    byte_vec: &mut Vec<u8>,
    bq_array: &mut ArrayViewMut2<u8>,
    header: &CBCLHeader,
    filter: &[bool],
    i: usize,
) -> () {
    if let Ok(uncomp_bytes) = extract_tiles(header, i) {
        // unpack the bytes, filtering out the reads that didn't pass
        byte_vec.extend(
            uncomp_bytes.iter()
                .zip(filter.chunks(2))
                .flat_map(|(v, f)| unpack_byte(v, f, header))
        );

        bq_array.assign(&ArrayView::from_shape(bq_array.raw_dim(), byte_vec).unwrap());
        byte_vec.clear();
    }
}


/// Create arrays of read and qscore values from a set of tiles
pub fn extract_reads(
    headers: &[CBCLHeader], filter: &[bool], pf_filter: &[bool], i: usize,
) -> Array3<u8> {
    let n_pf = pf_filter.iter().map(|&b| if b { 1 } else { 0 }).sum::<usize>();
    let n_cycles = headers.len();

    // preallocate a vector for bases/qscores
    let mut byte_vec = Vec::with_capacity(n_pf * 2);

    // preallocate an array for total output, with default values
    let mut out_array = Array3::zeros((n_cycles, n_pf, 2));

    out_array.index_axis_mut(Axis(2), 0).fill(b'N');
    out_array.index_axis_mut(Axis(2), 1).fill(b'#');

    for (mut row, h) in out_array.axis_iter_mut(Axis(0)).zip(headers) {
        let h_filter = if h.non_pf_clusters_excluded { pf_filter } else { filter };

        process_tiles(&mut byte_vec, &mut row, h, h_filter, i);
    }

    out_array
}


#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use ndarray::Array2;

    use crate::cbcl_header_decoder::cbcl_header_decoder;
    use crate::novaseq_run::NovaSeqRun;

    #[test]
    fn u8_to_base() {
        let expected_bases = vec![b'A', b'C', b'G', b'T', b'N'];
        let actual_bases: Vec<_> = [0, 1, 2, 3, 4].iter()
            .map(|&b| super::u8_to_base(b, 70))
            .collect();

        assert_eq!(actual_bases, expected_bases);
    }

    #[test]
    fn extract_tiles() {
        let cbcl_path = PathBuf::from("test_data/190414_A00111_0296_AHJCWWDSXX").join(
            "Data/Intensities/BaseCalls/L001/C1.1/L001_1.cbcl"
        );
        let cbcl_header = cbcl_header_decoder(&cbcl_path, 2).unwrap();

        let expected_bytes = vec![
            212, 254, 220, 221, 166, 108, 217, 232, 236, 221,
            157, 216, 220, 220, 205, 222, 140, 212, 157, 254,
            199, 221, 237, 185, 252, 199, 237, 253, 253, 68,
            237, 205, 199, 199, 237, 109, 205, 79, 200, 220,
            76, 253, 204, 253, 95, 223, 238, 78, 79, 206,
            220, 152, 220, 157, 255, 196, 207, 207, 133, 78,
            236, 222, 205, 254, 237, 204, 198, 218, 236, 204,
            206, 204, 214, 207, 222, 204, 201, 221, 103, 207,
            204, 196, 204, 88, 216, 205, 222, 251, 253, 206,
            206, 237, 223, 220, 205, 76, 220, 205, 232, 220
        ];

        let uncomp_bytes = super::extract_tiles(&cbcl_header, 0).unwrap();

        assert_eq!(uncomp_bytes, expected_bytes)
    }

    #[test]
    fn process_tiles() {
        let run_path = PathBuf::from("test_data/190414_A00111_0296_AHJCWWDSXX");
        let novaseq_run = NovaSeqRun::read_path(run_path, 2, false).unwrap();

        let expected_bq_pairs = vec![
            84, 70, 65, 70, 67, 70, 67, 70, 67, 70, 65, 70, 67, 58, 67, 70
        ];

        let header = &novaseq_run.headers.get(&(1, 1)).unwrap()[0];
        let filter = &novaseq_run.filters.get(&(1, 1)).unwrap()[0];

        let n_pf = filter.iter().map(|&b| if b { 1 } else { 0 }).sum();
        let mut byte_vec = Vec::with_capacity(n_pf * 2);
        let mut bq_array = Array2::zeros((n_pf, 2));

        super::process_tiles(
            &mut byte_vec, &mut bq_array.view_mut(), header, filter, 0
        );

        let bq_pairs: Vec<_> = bq_array.iter().cloned().take(16).collect();

        assert_eq!(bq_pairs, expected_bq_pairs)
    }
}
