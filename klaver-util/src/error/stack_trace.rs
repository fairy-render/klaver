use core::fmt;

use udled::{
    Char, EOF, Input, Next, Tokenizer, any, buffer::StringBuffer, prelude::*,
    tokenizers::WhiteSpace,
};
use udled_tokenizers::{Ident, Integer};

#[derive(Debug, Clone)]
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

    while !input.is(EOF) {
        input.eat(Next.until(LineParser.or(EOF)))?;

        if input.is(LineParser) {
            files.push(input.parse(LineParser)?);
        }
    }

    Ok(files)
}

struct LineParser;

impl<'input> Tokenizer<'input, StringBuffer<'input>> for LineParser {
    type Token = StackTrace;

    fn to_token(
        &self,
        reader: &mut udled::Reader<'_, 'input, StringBuffer<'input>>,
    ) -> Result<Self::Token, udled::Error> {
        let ws = WhiteSpace.many();

        reader.eat(("at", &ws))?;

        let func = if reader.is('<') {
            reader.parse("<anonymous>")?
        } else {
            reader.parse(Ident)?
        };

        reader.eat(&ws)?;

        let (file, line, column) = if reader.is('(') {
            reader.eat("(")?;

            let path = reader.parse((any!("./", "/", Char), Next.until(':')).slice())?;

            reader.eat(':')?;

            let (line, column) = reader.parse(LineColumn)?;

            reader.eat(")")?;

            (path.value.to_string(), line, column)
        } else {
            reader.eat(':')?;
            let (line, col) = reader.parse(LineColumn)?;
            ("".to_string(), line, col)
        };

        Ok(StackTrace {
            file,
            line,
            column,
            function: func.value.to_string(),
        })
    }

    fn peek<'a>(&self, reader: &mut udled::Reader<'_, 'input, StringBuffer<'input>>) -> bool {
        reader.is("at")
    }
}

pub struct LineColumn;

impl<'input> Tokenizer<'input, StringBuffer<'input>> for LineColumn {
    type Token = (u32, u32);

    fn to_token(
        &self,
        reader: &mut udled::Reader<'_, 'input, StringBuffer<'input>>,
    ) -> Result<Self::Token, udled::Error> {
        let line = reader.parse(Integer)?;
        reader.eat(':')?;
        let column = reader.parse(Integer)?;

        Ok((line.value as u32, column.value as u32))
    }
}
