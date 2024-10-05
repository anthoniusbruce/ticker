use crate::symbol_processor::symbol_processor::process_symbols;
use std::{
    cell::RefCell,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
use structopt::StructOpt;
use yahoo_finance_api::time::OffsetDateTime;

mod symbol_processor;
mod unit_tests;

thread_local! {static LOG_FILE_PATH:RefCell<Option<PathBuf>> = RefCell::new(None::<PathBuf>)}

/// Struct used to manager the command line inputs
#[derive(StructOpt)]
#[structopt(
    name = "ticker",
    about = "reads from the supplied file name, gets the last month of ticker data from yahoo finance and places the ticker information in the output file under the name of the ticker symbol"
)]
struct Opt {
    /// input file
    #[structopt(parse(from_os_str), required(true))]
    file_name: PathBuf,
    #[structopt(parse(from_os_str), required(true))]
    output: PathBuf,
    #[structopt(parse(from_os_str), required(true))]
    log_file: PathBuf,
}

/// The main method, entry point to the app
fn main() {
    let opt = Opt::from_args_safe();

    match opt {
        Ok(args) => {
            validate_args(&args.file_name, &args.output, &args.log_file);
            let log_file_path = args.log_file.clone();
            LOG_FILE_PATH.with(|path| *path.borrow_mut() = Some(log_file_path));
            let file_contents = read_file(&args.file_name);
            let symbols = get_ticker_symbols(&file_contents);
            process_symbols(symbols, &args.output);
        }
        Err(e) => println!("{e}"),
    }
}

/// convenience function to log trouble without interrupting things
fn log<T: std::fmt::Debug>(symbol: &str, info: T) {
    let timestamp = OffsetDateTime::now_utc();
    let message = format!(
        "TS: {} {}: {}: {:?}\n",
        timestamp.date(),
        timestamp.time(),
        symbol,
        info
    );
    LOG_FILE_PATH.with(|path| {
        let opt = path.borrow().clone();
        match opt {
            Some(log_file_path) => {
                let mut file = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(log_file_path)
                    .unwrap();
                file.write(message.as_bytes()).unwrap();
            }
            None => {
                println!("{}", message);
            }
        }
    });
}

/// Method to read the data from the file and return a string of the data
fn read_file(file_name: &PathBuf) -> String {
    let result = fs::read_to_string(file_name);
    match result {
        Ok(s) => return s,
        Err(_) => panic!("file_name cannot be opened"),
    }
}

/// Method that makes sure the file and directory exist and that the directory can be written to
fn validate_args(file_name: &PathBuf, output_dir: &PathBuf, log_file: &PathBuf) {
    let file_exists = Path::exists(file_name);
    if !file_exists {
        panic!("file_name does not exist");
    }

    let dir_exists = Path::exists(&output_dir);
    if !dir_exists {
        panic!("output directory does not exist");
    }

    match fs::metadata(output_dir) {
        Ok(md) => {
            if md.permissions().readonly() {
                panic!("you do not have permission to write to the output directory");
            }
        }
        Err(e) => panic!("{e}"),
    }

    let mut log = log_file.clone();
    let mut log_exists = Path::exists(&log);
    if !log_exists {
        log.pop();
        log_exists = Path::exists(&log);
    }
    if !log_exists {
        panic!("log directory does not exist");
    }

    match fs::metadata(log) {
        Ok(md) => {
            if md.permissions().readonly() {
                panic!("you do not have permission to the log file")
            }
        }
        Err(e) => panic!("{e}"),
    }
}

/// method to separate ticker symbols from a text string
fn get_ticker_symbols<'a>(test_data: &'a str) -> Vec<&'a str> {
    let mut ret = Vec::new();
    let symbols = test_data.split(',');
    for sym in symbols {
        let item = sym.trim();
        if item.is_empty() {
            continue;
        }
        ret.push(sym.trim());
    }

    ret
}
