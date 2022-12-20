#[allow(unused)]
use tailsome::IntoResult;
// use color_eyre::eyre::Result;
// type Result<T, E = Error> = std::result::Result<T, E>;
// use color_eyre::{eyre::eyre, Help};
// pub type Result<T, C = Error> = core::result::Result<T, error_stack::Report<C>>;
use error_stack::{IntoReport, Result, ResultExt};

fn parse_numbers(s: &str) -> Result<Vec<i32>, ErrorV2> {
    s.split('\n')
        .flat_map(|line| {
            line.split(' ').map(|word| {
                word.parse::<i32>()
                    .into_report()
                    .change_context(ErrorV2::ParseInt(word.to_owned()))
                    .attach_printable("hello")

                // .map_err(|e| eyre!(e).wrap_err("Failed to parse").suggestion("suggest").into())
                // .map_err(|e| e.parse_int(word) )
                // .map_err(|e| Error::ParseInt { input: word.to_owned(), source: e })
            })
        })
        .collect::<Result<Vec<i32>, _>>()
}

pub fn play() -> Result<(), ErrorV2> {
    let s = "1 2 3 4 s
5 6 7
8 9";
    parse_numbers(s)?;
    // dbg!(res);
    Ok(())
    // Err(eyre!("help").warning("Failed to parse").note("some note").wrap_err("another err").into())
}

#[allow(unused)]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    // #[error("Failed to parse '{input}'")]
    // ParseInt {
    //     input: String,
    //     #[source]
    //     source: std::num::ParseIntError,
    // },
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("bad")]
    Bad,
    #[error(transparent)]
    Other(#[from] color_eyre::eyre::Error),
    // #[error(transparent)]
    // Other(#[from] error_stack::)
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorV2 {
    #[error("Failed to parse as int: '{0}'")]
    ParseInt(String),
}

// impl Error {
//     fn parse_int(input: &str, e: std::num::ParseIntError) -> Self {
//         Self::ParseInt{input: input.to_owned(), source: e}
//     }
// }

// trait ParseIntExt {
//     fn parse_int(self, input: &str) -> Error;
// }

// impl ParseIntExt for std::num::ParseIntError {
//     fn parse_int(self, input: &str) -> Error {
//         Error::ParseInt{input: input.to_owned(), source: self}
//     }
// }
