use udled::{
    any,
    token::{Alphabetic, Char, Ident, Many, Opt, Ws},
    Input, Lex, Tokenizer, WithSpan,
};

const WS: Many<Ws> = Many(Ws);

#[derive(Debug)]
pub struct StackTrace {
    file: String,
    line: u32,
    column: u32,
    function: String,
}

pub fn parse(input: &str) -> Result<Vec<StackTrace>, udled::Error> {
    let mut input = Input::new(input);

    let mut files = Vec::default();

    while !input.eos() {
        input.eat(WS)?;

        if input.eos() {
            break;
        }

        if input.peek("at")? {
            files.push(input.parse(LineParser)?);
        } else {
            input.eat(Char)?;
        }
    }

    Ok(files)
}

// struct Char;

// impl Tokenizer for Char {
//     type Token<'a> = &'a str;

//     fn to_token<'a>(
//         &self,
//         reader: &mut udled::Reader<'_, 'a>,
//     ) -> Result<Self::Token<'a>, udled::Error> {
//         reader.eat_ch()
//     }
// }

struct LineParser;

impl Tokenizer for LineParser {
    type Token<'a> = StackTrace;

    fn to_token<'a>(
        &self,
        reader: &mut udled::Reader<'_, 'a>,
    ) -> Result<Self::Token<'a>, udled::Error> {
        reader.eat(("at", WS))?;

        let func = if reader.peek('<')? {
            let span = reader.parse("<anonymous>")?;
            Lex::new(span.slice(reader.input()).unwrap(), span)
        } else {
            reader.parse(Ident)?
        };

        reader.eat((WS, "("))?;

        let start = reader.parse(any!("./", "/", Char))?.span();

        loop {
            if reader.eof() {
                return Err(reader.error("Reached EOS"));
            }
        }

        todo!()
    }
}
