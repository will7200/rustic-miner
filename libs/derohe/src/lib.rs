#[macro_use] extern crate serde_derive;
extern crate serde;
pub mod rpc;
pub mod block;
pub mod pow;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
