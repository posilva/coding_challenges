extern crate getopts;

use getopts::Options;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

#[derive(Default)]
struct WCCmd {
    /// flag to enable the count of bytes
    bytes: bool,

    /// flag to enable the count of lines
    lines: bool,

    /// flag to enable the count of words
    words: bool,

    /// flag to enable the count of characters
    chars: bool,

    /// input files to be processed (empty if stdin)
    inputs: Vec<WCInput>,

    /// output processed
    outputs: Vec<WCOutput>,
}

impl WCCmd {
    fn show(&self) {
        for o in &self.outputs {
            println!("{}", o.as_string(self))
        }
    }

    fn use_default_flags(&self) -> bool {
        !self.chars && !self.lines && !self.words && !self.bytes
    }

    fn from_args(args: Vec<String>) -> Result<Self, getopts::Fail> {
        let mut opts = Options::new();

        // define the options
        opts.optflag("c", "bytes", "count the number of bytes");
        opts.optflag("l", "lines", "count the number of lines");
        opts.optflag("w", "words", "count the number of words");
        opts.optflag("m", "chars", "count the number of chars");

        // parse the options
        let opts_matches = opts.parse(&args[1..])?;
        let arg_inputs = opts_matches.free.to_owned();
        let mut parsed_inputs = Vec::new();

        if !arg_inputs.is_empty() {
            for f in arg_inputs {
                parsed_inputs.push(WCInput::File(f.to_string()));
            }
        } else {
            parsed_inputs.push(WCInput::StdIn())
        }

        Ok(WCCmd {
            bytes: opts_matches.opt_present("c"),
            lines: opts_matches.opt_present("l"),
            words: opts_matches.opt_present("w"),
            chars: opts_matches.opt_present("m"),
            inputs: parsed_inputs,
            outputs: Vec::new(),
        })
    }

    fn process(&mut self) -> std::io::Result<()> {
        let mut buffer: Vec<u8> = Vec::new();
        let default = self.use_default_flags();

        for input in &mut self.inputs {
            let mut output = WCOutput::default();

            input.as_buffer(&mut buffer)?;
            output.filename = input.path();

            let mut parsing_word = true;
            for b in buffer.iter() {
                if self.bytes || default {
                    output.byte_ct += 1;
                }
                if (self.lines || default) && *b == b'\n' {
                    output.line_ct += 1;
                }

                if self.words || default {
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

            if self.chars {
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
            self.outputs.push(output);
            buffer.clear();
        }
        Ok(())
    }
}

enum WCInput {
    File(String),
    StdIn(),
}

impl WCInput {
    fn path(&self) -> Option<String> {
        match self {
            WCInput::File(s) => Some(s.clone()),
            WCInput::StdIn() => None,
        }
    }

    fn as_buffer(&mut self, buffer: &mut Vec<u8>) -> std::io::Result<()> {
        match self {
            WCInput::File(f) => {
                let file = File::open(f)?;
                let mut reader = BufReader::new(file);
                reader.read_to_end(buffer)?;
            }
            WCInput::StdIn() => {
                std::io::stdin().read_to_end(buffer)?;
            }
        }
        Ok(())
    }
}

#[derive(Default)]
struct WCOutput {
    byte_ct: u64,
    line_ct: u64,
    word_ct: u64,
    char_ct: u64,
    filename: Option<String>,
}

impl WCOutput {
    fn as_string(&self, wc: &WCCmd) -> String {
        let mut out = String::new();
        let default = wc.use_default_flags();

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
        if let Some(f) = &self.filename {
            out.push_str(format!(" {}", f).as_str())
        }
        out.to_string()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect();
    let mut wc = WCCmd::from_args(args)?;
    wc.process()?;
    wc.show();
    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::{WCCmd, WCInput, WCOutput};

    #[test]
    fn wccmd_parse_env_args_test() {
        let args = vec![String::from("ccwc"), String::from("test.file")];
        let wc = WCCmd::from_args(args).unwrap();

        _ = wc;
    }

    #[test]
    fn input_path_test() {
        let filepath = String::from("test.file");
        let file_input = WCInput::File(filepath.clone());

        let stdin_input = WCInput::StdIn();
        assert_eq!(filepath, file_input.path().unwrap());
        assert_eq!(None, stdin_input.path());
    }

    #[test]
    fn wcoutput_print_using_default_test() {
        let wc = WCCmd::default();

        let out = WCOutput::default();
        let result = out.as_string(&wc);
        let expected = "\t0\t0\t0";
        assert_eq!(result, expected);
    }

    #[test]
    fn wcoutput_print_using_chars_test() {
        let mut wc = WCCmd::default();
        wc.chars = true;

        let mut out = WCOutput::default();
        out.char_ct = 123;

        let result = out.as_string(&wc);
        let expected = "\t123";
        assert_eq!(result, expected);
    }

    #[test]
    fn wcoutput_print_with_input_file_test() {
        let input = Some(String::from("./test_file.txt"));
        let wc = WCCmd::default();
        let mut out = WCOutput::default();
        out.filename = input.clone();

        let result = out.as_string(&wc);
        let expected = format!("\t0\t0\t0 {}", input.unwrap());
        assert_eq!(result, expected);
    }
}
