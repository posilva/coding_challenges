use argh::FromArgs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Read;

#[derive(FromArgs)]
#[argh(description = "ccwc – word, line, character, and byte count Word")]
struct WCArgs {
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

impl WCArgs {
    fn use_default(&self) -> bool {
        !self.chars && !self.lines && !self.words && !self.bytes
    }
}

enum WCInput {
    File(File, String),
    StdIn(io::Stdin, String),
}

impl WCInput {
    fn path(&self) -> &String {
        match self {
            WCInput::File(_, s) => s,
            WCInput::StdIn(_, s) => s,
        }
    }

    fn as_buffer(&mut self, buffer: &mut Vec<u8>) -> io::Result<()> {
        match self {
            WCInput::File(f, _) => {
                let mut reader = BufReader::new(f);
                reader.read_to_end(buffer)?;
            }
            WCInput::StdIn(s, _) => {
                s.read_to_end(buffer)?;
            }
        }
        Ok(())
    }
}

struct WCOutput {
    byte_ct: u64,
    line_ct: u64,
    word_ct: u64,
    char_ct: u64,
    filename: String,
}

impl WCOutput {
    fn as_string(&self, wc: &WCArgs) -> String {
        let mut out = String::new();
        let default = wc.use_default();

        if wc.lines || default {
            out.push_str(format!("\t{}", self.line_ct).as_str());
        }
        if wc.words || default {
            out.push_str(format!("\t{}", self.word_ct).as_str());
        }
        if wc.chars {
            out.push_str(format!("\t{}", self.char_ct).as_str());
        } else if wc.bytes || default {
            out.push_str(format!("\t{}", self.byte_ct).as_str());
        }
        if self.filename != "-" {
            out.push_str(format!(" {}", self.filename).as_str())
        }
        out.to_string()
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

fn parse_inputs(wc: &WCArgs, inputs: &mut Vec<WCInput>) -> io::Result<()> {
    if !wc.inputs.is_empty() {
        for f in &wc.inputs {
            inputs.push(WCInput::File(File::open(f)?, f.to_string()));
        }
    } else {
        inputs.push(WCInput::StdIn(io::stdin(), String::from("")))
    }
    Ok(())
}

fn process_input(input: &mut WCInput, wc: &WCArgs, buffer: &mut Vec<u8>) -> io::Result<WCOutput> {
    let mut output = WCOutput::default();
    let default = wc.use_default();

    input.as_buffer(buffer)?;
    output.filename = input.path().to_owned();

    let mut parsing_word = true;

    for b in buffer.iter() {
        if wc.bytes || default {
            output.byte_ct += 1;
        }
        if (wc.lines || default) && *b == b'\n' {
            output.line_ct += 1;
        }

        if wc.words || default {
            if parsing_word {
                if b.is_ascii_whitespace() {
                    output.word_ct += 1;
                    parsing_word = false;
                }
            } else if !b.is_ascii_whitespace() {
                parsing_word = true;
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
    let wc: WCArgs = argh::from_env();

    let mut inputs: Vec<WCInput> = Vec::new();
    parse_inputs(&wc, &mut inputs)?;

    let mut outputs = Vec::new();
    let mut buffer: Vec<u8> = Vec::new();
    for input in &mut inputs {
        let output = process_input(input, &wc, &mut buffer)?;
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
    use crate::{WCArgs, WCInput, WCOutput};

    #[test]
    fn input_path_test() {
        let filepath = String::from("test.file");
        let f = std::fs::File::create(filepath.clone()).expect("failed to create test file");
        let file_input = WCInput::File(f, filepath.clone());

        let stdin_input = WCInput::StdIn(std::io::stdin(), "-".to_owned());
        assert_eq!(filepath.clone().as_str(), file_input.path());
        assert_eq!(String::from("-").as_str(), stdin_input.path());
    }

    #[test]
    fn wcoutput_print_using_default_test() {
        let wc = WCArgs {
            bytes: false,
            lines: false,
            words: false,
            chars: false,
            inputs: Vec::new(),
        };

        let out = WCOutput::default();
        let result = out.as_string(&wc);
        let expected = "\t0\t0\t0";
        assert_eq!(result, expected);
    }

    #[test]
    fn wcoutput_print_using_chars_test() {
        let wc = WCArgs {
            bytes: false,
            lines: false,
            words: false,
            chars: true,
            inputs: Vec::new(),
        };

        let mut out = WCOutput::default();
        out.char_ct = 123;
        let result = out.as_string(&wc);
        let expected = "\t123";
        assert_eq!(result, expected);
    }

    #[test]
    fn wcoutput_print_with_input_file_test() {
        let input = String::from("./test_file.txt");
        let wc = WCArgs {
            bytes: false,
            lines: false,
            words: false,
            chars: false,
            inputs: Vec::new(),
        };

        let mut out = WCOutput::default();
        out.filename = input.clone();

        let result = out.as_string(&wc);
        let expected = format!("\t0\t0\t0 {}", input);
        assert_eq!(result, expected);
    }
}
