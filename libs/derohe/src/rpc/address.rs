use std::fmt::format;
use std::num::IntErrorKind::Empty;
use std::ops::Neg;

use bech32::{self, FromBase32, ToBase32, u5, Variant};
use num_bigint::{BigInt, BigUint, Sign};

use bn::{AffineG1, CurveError, Fq, G1};
use bn::arith::U256;

use crate::rpc::rpc::Arguments;

#[derive(Debug, Clone, PartialEq)]
pub struct Address {
    network: u64,
    mainnet: bool,
    proof: bool,
    public_key: Option<G1>,
    arguments: Arguments,
}

pub trait ReversedG1 {
    fn from_compressed_reversed_sign(bytes: &[u8]) -> Result<G1, CurveError>;
}

impl ReversedG1 for G1 {
    fn from_compressed_reversed_sign(bytes: &[u8]) -> Result<Self, CurveError> {
        if bytes.len() != 33 { return Err(CurveError::InvalidEncoding); }

        let sign = bytes[32];
        let fq = Fq::from_slice(&bytes[0..32])?;
        let x = fq;
        let y_squared = (fq * fq * fq) + Self::b();

        let y = y_squared.sqrt().ok_or(CurveError::NotMember)?;
        // if sign == 2 && y.into_u256().get_bit(0).expect("bit 0 always exist; qed") { y = y.neg(); }
        // else if sign == 3 && !y.into_u256().get_bit(0).expect("bit 0 always exist; qed") { y = y.neg(); }
        // else if sign != 3 && sign != 2 {
        //     return Err(CurveError::InvalidEncoding);
        // }
        // println!("y= {:#?}",y.into_u256());
        AffineG1::new(x, y).map_err(|_| CurveError::NotMember).map(Into::into)
    }
}

impl Address {
    pub fn from_string(encoded: &str) -> Result<Self, String> {
        let (hrp, data, variant): (String, Vec<u5>, Variant) = match bech32::decode(encoded) {
            Ok(val_) => val_,
            Err(e) => return Err(e.to_string())
        };
        let mut address = Address {
            network: 0,
            mainnet: false,
            proof: false,
            public_key: None,
            arguments: Arguments::empty(),
        };

        match hrp.as_str() {
            "dero" | "deroi" | "deto" | "detoi" | "deroproof" => (),
            _ => return Err("invalid human-readable part".parse().unwrap())
        };

        if data.len() < 1 {
            return Err(format!("invalid decode version: {}", data.len()));
        }

        let res: Vec<u8> = Vec::<u8>::from_base32(&data).unwrap();

        if res[0] != 1 {
            return Err(format!("invalid address version: {}", res[0]));
        }
        let res_bytes = &res[1..];
        if res_bytes.len() < 33 {
            return Err(format!("invalid address length as per spec: {}", res_bytes.len()));
        }
        address.public_key = match G1::from_compressed_reversed_sign(&res_bytes[..33]) {
            Ok(g) => Some(g),
            Err(e) => return Err(format!("{:?}", e))
        };
        address.mainnet = match hrp.as_str() {
            "deto" | "detoi" => false,
            _ => true
        };
        if hrp.as_str() == "deroproof" {
            address.proof = true
        }
        // TODO
        // 	switch {
        // 	case len(res) == 33 && (dechrp == "dero" || dechrp == "deto"):
        // 	case (dechrp == "deroi" || dechrp == "detoi" || dechrp == "deroproof"): // address contains service request
        // 		if err = result.Arguments.UnmarshalBinary(resbytes[33:]); err != nil {
        // 			return err
        // 		}
        // 	default:
        // 		return fmt.Errorf("invalid address length as per spec : %d", len(res))
        // 	}

        Ok(address)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::rpc::address::Address;

    #[test]
    fn valid_address() {
        let address = Address::from_string("dero1qyp5p7r2tqad8fvylrp4xp9l9q499kcpglvq4frw0pd4qtrtsa5d5qqx7tpmn").unwrap();
        assert!(address.public_key.is_some());
        assert_eq!(address.mainnet, true);
    }

    #[rstest]
    #[case(String::from("der1qyp5p7r2tqad8fvylrp4xp9l9q499kcpglvq4frw0pd4qtrtsa5d5qqx7tpmn"))]
    #[case(String::from("der1qyp5p7r2tqad8fvylrp4xp9l9q499kcpglvq4frw0pd4qtrtsa5d5qqx7tpmm"))]
    fn invalid_address(#[case] address: String) {
        let result = Address::from_string(address.as_str());
        match result {
            Ok(_v) => assert!(true, "{}", "shouldn't be a valid address"),
            Err(_) => ()
        }
    }
}