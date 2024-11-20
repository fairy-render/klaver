use core::fmt;

use udled::{
    any,
    token::{Char, Ident, Int, Many, Ws},
    Input, Lex, Span, Tokenizer, WithSpan,
};

const WS: Many<Ws> = Many(Ws);

#[derive(Debug)]
pub struct StackTrace {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub function: String,
}

impl fmt::Display for StackTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}:{}:{})",
            self.function, self.file, self.line, self.column
        )
    }
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

        let path_start = reader.parse(any!("./", "/", Char))?.span();

        let path_end = loop {
            if reader.eof() {
                return Err(reader.error("Reached EOS"));
            }

            if reader.peek(':')? {
                let end = reader.position();
                reader.eat(':')?;
                break end;
            }

            reader.eat_ch()?;
        };

        let fn_span = Span::new(path_start.start, path_end);

        let line = reader.parse(Int)?;
        reader.eat(':')?;
        let column = reader.parse(Int)?;

        reader.eat(")")?;

        Ok(StackTrace {
            file: fn_span.slice(reader.input()).unwrap().to_string(),
            line: line.value as u32,
            column: column.value as u32,
            function: func.as_str().to_string(),
        })
    }

    fn peek<'a>(&self, reader: &mut udled::Reader<'_, '_>) -> Result<bool, udled::Error> {
        reader.peek("as")
    }
}
