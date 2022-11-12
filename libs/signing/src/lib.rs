#![deny(clippy::all, clippy::nursery)]
#![warn(clippy::pedantic)]

use std::io::Write;

use rsa::BigUint;

mod error;
mod private;
mod public;
mod signature;

pub use error::Error;
pub use private::BIPrivateKey;

/// Writes a ``BigUint`` to the given output.
///
/// # Errors
/// If the output fails to write.
pub fn write_biguint<O: Write>(output: &mut O, bn: &BigUint, size: usize) -> Result<(), Error> {
    let mut vec: Vec<u8> = bn.to_bytes_le();
    vec.resize(size, 0);
    output.write_all(&vec).map_err(std::convert::Into::into)
}