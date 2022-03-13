use std::convert::TryInto;

use sha3::{Digest, Sha3_256};

use super::salsa20;

pub const STAGE1_LENGTH: usize = 9973;

pub fn pow16(input: &[u8]) -> [u8; 32] {
    let mut key = sha3(&input); // Step 1: calculate SHA3 of input data
    let mut stage1_result = [0u8; STAGE1_LENGTH];

    salsa20::xor_key_stream(
        &mut stage1_result,
        &[0u8; STAGE1_LENGTH],
        &key,
    );
    let mut sa = vec![0; stage1_result.len()];
    // divsufsort::sort_in_place(&stage1_result, &mut sa);
    cdivsufsort::sort_in_place(&stage1_result, &mut sa);
    let val: Vec<u16> = sa.iter().map(|&val| val as u16).collect();
    let bb = unsafe { val.align_to::<u8>().1 };
    let key = sha3(bb);
    key.into()
}

pub fn sha3(input: &[u8]) -> [u8; 32] {
    let mut output: [u8; 32] = [0; 32];
    let mut hasher = Sha3_256::new();
    hasher.update(input);

    output.copy_from_slice(hasher.finalize().as_slice());
    output
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use crate::pow::astrobwt::pow16;

    use super::{sha3, STAGE1_LENGTH};
    use super::salsa20;
    use bn::arith;

    #[rstest]
    fn test_pow16() {
        let input: [u8; 48] = [65, 90, 158, 0, 0, 0, 131, 134, 179, 254, 154, 24, 0, 0, 0, 0, 76, 45, 130, 143, 5, 131, 168, 109, 185, 99, 157, 54, 84, 143, 129, 113, 0, 0, 0, 0, 222, 179, 70, 94, 29, 49, 111, 0, 0, 0, 2, 1];
        let result =  pow16(&input);
        assert_eq!(result, [129, 80, 247, 57, 240, 97, 71, 68, 66, 61, 172, 6, 56, 169, 252, 234, 128, 202, 217, 52, 88, 75, 175, 166, 165, 225, 153, 165, 170, 45, 122, 61]);
        let value = num_bigint::BigInt::from_bytes_le(num_bigint::Sign::Plus, &result);
        println!("{}", value);
    }
}