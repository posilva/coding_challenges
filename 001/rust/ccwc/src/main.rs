use argh::FromArgs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Read;

#[derive(FromArgs)]
#[argh(description = "ccwc – word, line, character, and byte count Word")]
struct WC {
    /// flag to enable the count of bytes
    #[argh(switch, short = 'c')]
    bytes: bool,

    /// flag to enable the count of lines
    #[argh(switch, short = 'l')]
    lines: bool,

    /// flag to enable the count of words
    #[argh(switch, short = 'w')]
    words: bool,

    /// flag to enable the count of characters
    #[argh(switch, short = 'm')]
    chars: bool,

    #[argh(positional, greedy, description = "input file(s)")]
    inputs: Vec<String>,
}

enum Input {
    File(File, String),
    StdIn(io::Stdin, String),
}

struct WCOutput {
    byte_ct: u64,
    line_ct: u64,
    word_ct: u64,
    char_ct: u64,
    filename: String,
}

impl WCOutput {
    fn as_string(&self, wc: &WC) -> String {
        let mut out = String::new();
        let default = !wc.chars && !wc.lines && !wc.words && !wc.bytes;

        if wc.lines || default {
            out.push_str(String::from(format!("\t{}", self.line_ct)).as_str());
        }
        if wc.words || default {
            out.push_str(String::from(format!("\t{}", self.word_ct)).as_str());
        }
        if wc.chars {
            out.push_str(String::from(format!("\t{}", self.char_ct)).as_str());
        } else if wc.bytes || default {
            out.push_str(String::from(format!("\t{}", self.byte_ct)).as_str());
        }
        if self.filename != "-" {
            out.push_str(format!(" {}", self.filename).as_str())
        }
        format!("{}", out)
    }
}

impl Default for WCOutput {
    fn default() -> Self {
        WCOutput {
            byte_ct: 0,
            line_ct: 0,
            word_ct: 0,
            char_ct: 0,
            filename: String::from("-"),
        }
    }
}

impl Input {
    fn path(&self) -> &String {
        match self {
            Input::File(_, s) => s,
            Input::StdIn(_, s) => s,
        }
    }

    fn to_buffer(&mut self, buffer: &mut Vec<u8>) -> io::Result<()> {
        match self {
            Input::File(f, _) => {
                let mut reader = BufReader::new(f);
                reader.read_to_end(buffer)?;
            }
            Input::StdIn(s, _) => {
                s.read_to_end(buffer)?;
            }
        }
        Ok(())
    }
}

fn process_inputs(wc: &WC, inputs: &mut Vec<Input>) -> io::Result<()> {
    if wc.inputs.len() > 0 {
        for f in &wc.inputs {
            inputs.push(Input::File(File::open(f)?, f.to_string()));
        }
    } else {
        inputs.push(Input::StdIn(io::stdin(), String::from("")))
    }
    Ok(())
}

fn result_from_input(input: &mut Input, wc: &WC, buffer: &mut Vec<u8>) -> io::Result<WCOutput> {
    let mut output = WCOutput::default();
    // TODO: this can be made only once when parsing inputs
    // We can make a kind of pipeline where the byte is visited by
    // different parses and outputs the result in the end
    let default = !wc.chars && !wc.lines && !wc.words && !wc.bytes;

    input.to_buffer(buffer)?;
    output.filename = input.path().to_owned();

    let mut parsing_word = true;

    for b in buffer.iter() {
        if wc.bytes || default {
            output.byte_ct += 1;
        }
        if wc.lines || default {
            if *b == b'\n' {
                output.line_ct += 1;
            }
        }

        if wc.words || default {
            if parsing_word {
                if b.is_ascii_whitespace() {
                    output.word_ct += 1;
                    parsing_word = false;
                }
            } else {
                if !b.is_ascii_whitespace() {
                    parsing_word = true;
                }
            }
        }
    }

    if wc.chars {
        match String::from_utf8(buffer.to_owned()) {
            Ok(s) => {
                output.char_ct = s.chars().count() as u64;
            }
            _ => {
                // if there is an error we fallback for bytes
                output.char_ct = output.byte_ct;
            }
        }
    }

    Ok(output)
}

fn main() -> io::Result<()> {
    let wc: WC = argh::from_env();

    let mut inputs: Vec<Input> = Vec::new();
    process_inputs(&wc, &mut inputs)?;

    let mut outputs = Vec::new();
    let mut buffer: Vec<u8> = Vec::new();
    for input in &mut inputs {
        let output = result_from_input(input, &wc, &mut buffer)?;
        outputs.push(output);
        buffer.clear();
    }

    for o in outputs {
        println!("{}", o.as_string(&wc));
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn process_inputs_test() {
        assert_eq!(0, 0)
    }

    #[test]
    fn input_path_test() {
        assert_eq!(0, 0)
    }
}
