use argh::FromArgs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Read;

#[derive(FromArgs)]
#[argh(description = "ccwc – word, line, character, and byte count Word")]
struct WC {
    #[argh(switch, short = 'c', description = "count the number of bytes")]
    bytes: bool,

    #[argh(positional, greedy, description = "input file(s)")]
    inputs: Vec<String>,
}

enum Input {
    File(File, String),
    StdIn(io::Stdin, String),
}

impl Input {
    fn source(&self) -> &String {
        match self {
            Input::File(_, s) => s,
            Input::StdIn(_, s) => s,
        }
    }
}
fn process_inputs(wc: &WC, inputs: &mut Vec<Input>) -> io::Result<()> {
    if wc.inputs.len() > 0 {
        // here we are iterating input argument by reference
        for f in &wc.inputs {
            inputs.push(Input::File(File::open(f)?, f.to_string()));
        }
    } else {
        inputs.push(Input::StdIn(io::stdin(), String::from("")))
    }
    Ok(())
}

fn read_inputs(input: &mut Input, buffer: &mut Vec<u8>) -> io::Result<()> {
    match input {
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

fn main() -> io::Result<()> {
    let wc: WC = argh::from_env();

    let mut inputs: Vec<Input> = Vec::new();
    process_inputs(&wc, &mut inputs)?;

    let mut buffer: Vec<u8> = Vec::new();
    for mut input in &mut inputs {
        read_inputs(&mut input, &mut buffer)?;
        println!("{}\t{}", buffer.len(), input.source());
        buffer.clear();
    }

    Ok(())
}
