use play::{play, Error, ErrorV2};

// use color_eyre::eyre::Result;
// use color_eyre::eyre::eyre;
// type Result<T, E = Error> = std::result::Result<T, E>;
use error_stack::Result;

fn main() -> Result<(), ErrorV2> {
    // color_eyre::install()?;
    play()?;
    // res.map_err(|e| {
    //     match e {
    //         Error::Other(inner_e) => {
    //             println!("hi");
    //             inner_e
    //         },
    //         _ => eyre!(e),
    //     }
    // })?;
    Ok(())
}
