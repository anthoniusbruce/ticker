use csv::Writer;
use std::{
    cell::RefCell,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
use structopt::StructOpt;
use tokio::runtime::Builder;
use yahoo_finance_api::{time::OffsetDateTime, Quote};

thread_local! {static LOG_FILE_PATH:RefCell<Option<PathBuf>> = RefCell::new(None::<PathBuf>)}

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

/// using the list of symbols get the daily quotes for the past month
fn process_symbols(symbols: Vec<&str>, output_dir: &PathBuf) {
    for symbol in symbols {
        let one_day = time::Duration::days(1);
        let one_month = time::Duration::days(30);
        let today = OffsetDateTime::now_utc();
        let end_date = today - one_day;
        let start_date = end_date - one_month;
        let quotes = get_quotes(symbol, &start_date, &end_date);
        let mut gains = Vec::new();
        for quote in quotes {
            let gain = get_gain(quote);
            gains.push(gain);
        }
        save_gains(output_dir, symbol, gains);
    }
}

/// saves the daily gains to a csv file in the output directory
fn save_gains(output_dir: &PathBuf, symbol: &str, gains: Vec<f64>) {
    let file_name = output_dir.join(symbol);
    let file_result = File::create(file_name);
    match file_result {
        Err(e) => log(e),
        Ok(file) => {
            let mut writer = Writer::from_writer(file);
            let serialize_result = writer.serialize(gains);
            match serialize_result {
                Err(e) => log(e),
                Ok(_) => (),
            }
            let flush_result = writer.flush();
            match flush_result {
                Err(e) => log(e),
                Ok(_) => (),
            }
        }
    }
}

/// converts Quote to the single value of the gain of the day (+/-)
fn get_gain(quote: Quote) -> f64 {
    quote.close - quote.open
}

/// convenience function to log trouble without interrupting things
fn log<T: std::fmt::Debug>(info: T) {
    let timestamp = OffsetDateTime::now_utc();
    let message = format!(
        "TS: {} {}: {:?}\n",
        timestamp.date(),
        timestamp.time(),
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

/// Method to get that quotes over a duration for a given ticker symbol
fn get_quotes(symbol: &str, start: &OffsetDateTime, end: &OffsetDateTime) -> Vec<Quote> {
    let quotes = Vec::new();
    let provider_result = yahoo_finance_api::YahooConnector::new();
    match provider_result {
        Err(e) => log(e),
        Ok(provider) => {
            let builder_result = Builder::new_current_thread().enable_all().build();
            match builder_result {
                Err(e) => log(e),
                Ok(builder) => {
                    let resp_result =
                        builder.block_on(provider.get_quote_history(symbol, *start, *end));
                    match resp_result {
                        Err(e) => log(e),
                        Ok(resp) => match resp.quotes() {
                            Err(e) => log(e),
                            Ok(response) => {
                                let message = format!(
                                    "Success: {} {} - {}",
                                    symbol,
                                    start.date(),
                                    end.date(),
                                );
                                log(message);
                                return response;
                            }
                        },
                    }
                }
            }
        }
    }
    quotes
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

/// Tests
#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn read_test_data() -> String {
        let read_result = fs::read_to_string("testdata.txt");
        match read_result {
            Ok(s) => return s,
            Err(e) => panic!("file read error: {e}"),
        };
    }

    fn vectors_are_equal(v1: Vec<&str>, v2: Vec<&str>) -> bool {
        if v1.iter().count() != v2.iter().count() {
            println!(
                "counts are not equal v1={} v2 = {}",
                v1.iter().count(),
                v2.iter().count()
            );
            return false;
        }

        for s in v1.iter() {
            if !v2.contains(&s) {
                println!("v2 search found no {s}");
                return false;
            }
        }

        for s in v2.iter() {
            if !v1.contains(&s) {
                println!("v1 search found no {s}");
                return false;
            }
        }

        return true;
    }

    #[test]
    fn get_gain_positive_value() {
        // assign
        let quote = Quote {
            close: 1.0,
            open: 0.5,
            timestamp: 0,
            high: 0.0,
            low: 0.0,
            volume: 0,
            adjclose: 0.0,
        };
        let expected = 0.5;

        // act
        let actual = get_gain(quote);

        // assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn get_gain_negative_value() {
        // assign
        let quote = Quote {
            close: 1.0,
            open: 2.5,
            timestamp: 0,
            high: 0.0,
            low: 0.0,
            volume: 0,
            adjclose: 0.0,
        };
        let expected = -1.5;

        // act
        let actual = get_gain(quote);

        // assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn get_gain_no_movement() {
        // assign
        let quote = Quote {
            close: 1.0,
            open: 1.0,
            timestamp: 0,
            high: 0.0,
            low: 0.0,
            volume: 0,
            adjclose: 0.0,
        };
        let expected = 0.0;

        // act
        let actual = get_gain(quote);

        // assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn get_gain_zeroes() {
        // assign
        let quote = Quote {
            close: 0.0,
            open: 0.0,
            timestamp: 0,
            high: 0.0,
            low: 0.0,
            volume: 0,
            adjclose: 0.0,
        };
        let expected = 0.0;

        // act
        let actual = get_gain(quote);

        // assert
        assert_eq!(expected, actual);
    }

    #[test]
    #[should_panic(expected = "file_name cannot be opened")]
    fn main_invalid_file_panics() {
        // assign
        let file_name = PathBuf::from(".");

        // act
        read_file(&file_name);

        // assert
        assert!(false);
    }

    #[test]
    fn validate_args_good_file_and_directory_and_log() {
        // assign
        let file_name = PathBuf::from("testdata.txt");
        let dir = PathBuf::from(".");
        let log = PathBuf::from("testdata.txt");

        // act
        validate_args(&file_name, &dir, &log);

        // assert
        assert!(true);
    }

    #[test]
    #[should_panic(expected = "file_name does not exist")]
    fn validate_args_file_does_not_exist() {
        // assign
        let file_name = PathBuf::from("badtestdata.txt");
        let dir = PathBuf::from(".");
        let log = PathBuf::from("testdata.txt");

        // act
        validate_args(&file_name, &dir, &log);

        // assert
        assert!(false)
    }

    #[test]
    #[should_panic(expected = "output directory does not exist")]
    fn validate_args_directory_does_not_exist() {
        // assign
        let file_name = PathBuf::from("testdata.txt");
        let dir = PathBuf::from("baddirectory");
        let log = PathBuf::from("testdata.txt");

        // act
        validate_args(&file_name, &dir, &log);

        // assert
        assert!(false);
    }

    #[test]
    #[should_panic(expected = "you do not have permission to write to the output directory")]
    fn validate_args_directory_does_not_have_permissions() {
        // assign
        let file_name = PathBuf::from("testdata.txt");
        let dir = PathBuf::from("/dev");
        let log = PathBuf::from("testdata.txt");

        // act
        validate_args(&file_name, &dir, &log);

        // assert
        assert!(false);
    }

    #[test]
    #[should_panic(expected = "log directory does not exist")]
    fn validate_args_log_dir_does_not_exist() {
        // assign
        let file_name = PathBuf::from("testdata.txt");
        let dir = PathBuf::from(".");
        let log = PathBuf::from("noexist/noexist.txt");

        // act
        validate_args(&file_name, &dir, &log);

        // assert
        assert!(false);
    }

    #[test]
    #[should_panic(expected = "log directory does not exist")]
    fn validate_args_no_log_dir() {
        // assign
        let file_name = PathBuf::from("testdata.txt");
        let dir = PathBuf::from(".");
        let log = PathBuf::from("noexist.txt");

        // act
        validate_args(&file_name, &dir, &log);

        // assert
        assert!(false);
    }

    #[test]
    fn validate_args_good_file_and_directory_and_no_log_file() {
        // assign
        let file_name = PathBuf::from("testdata.txt");
        let dir = PathBuf::from(".");
        let log = PathBuf::from("./noexist.txt");

        // act
        validate_args(&file_name, &dir, &log);

        // assert
        assert!(true);
    }

    #[test]
    #[should_panic(expected = "you do not have permission to the log file")]
    fn validate_args_log_file_does_not_have_permissions() {
        // assign
        let file_name = PathBuf::from("testdata.txt");
        let dir = PathBuf::from(".");
        let log = PathBuf::from("readonly.txt");

        // act
        validate_args(&file_name, &dir, &log);

        // assert
        assert!(false);
    }

    #[test]
    #[should_panic(expected = "you do not have permission to the log file")]
    fn validate_args_log_directory_does_not_have_permissions() {
        // assign
        let file_name = PathBuf::from("testdata.txt");
        let dir = PathBuf::from(".");
        let log = PathBuf::from("/dev/log.txt");

        // act
        validate_args(&file_name, &dir, &log);

        // assert
        assert!(false);
    }

    #[test]
    fn get_ticker_symbols_11_items() {
        // assign
        let test_data = read_test_data();
        let expected = vec![
            "AACG", "AADI", "AADR", "AAL", "AAME", "AAOI", "AAON", "AAPB", "AAPD", "AAPL",
        ];

        // act
        let actual = get_ticker_symbols(&test_data);

        // assert
        assert!(vectors_are_equal(expected, actual), "vectors are not equal");
    }

    #[test]
    fn get_ticker_symbols_empty_string() {
        // assign
        let test_data = "";
        let expected = Vec::new();

        // act
        let actual = get_ticker_symbols(test_data);

        // assert
        assert!(vectors_are_equal(expected, actual));
    }

    #[test]
    fn get_ticker_symbols_one_item() {
        // assign
        let test_data = "AACG";
        let expected = vec!["AACG"];

        // act
        let actual = get_ticker_symbols(test_data);

        // assert
        assert!(vectors_are_equal(expected, actual));
    }

    #[test]
    fn get_ticker_symbols_many_items_with_extra_newlines() {
        // assign
        let test_data = "\nAACG,AADI,\n,\n,\n,AADR";
        let expected = vec!["AACG", "AADI", "AADR"];

        // act
        let actual = get_ticker_symbols(test_data);

        // assert
        assert!(vectors_are_equal(expected, actual));
    }
}
