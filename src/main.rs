use std::{
    fs::{self},
    path::{Path, PathBuf},
};
use structopt::StructOpt;
use tokio::runtime::Builder;
use yahoo_finance_api::{time::OffsetDateTime, Quote, YahooError};

/// The main method, entry point to the app
fn main() {
    let opt = Opt::from_args_safe();

    match opt {
        Ok(args) => {
            validate_args(&args.file_name, &args.output);
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
        println!("{symbol}:\n{:?}", quotes);
    }
}

/// Method to get that quotes over a duration for a given ticker symbol
fn get_quotes(
    symbol: &str,
    start: &OffsetDateTime,
    end: &OffsetDateTime,
) -> Result<Vec<Quote>, YahooError> {
    let provider = yahoo_finance_api::YahooConnector::new()?;
    let resp = Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(provider.get_quote_history(symbol, *start, *end))?;

    Ok(resp.quotes()?)
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
fn validate_args(file_name: &PathBuf, output_dir: &PathBuf) {
    let file_exists = Path::exists(file_name);
    if !file_exists {
        panic!("file_name does not exist");
    }

    let dir_exists = Path::exists(&output_dir);
    if !dir_exists {
        panic!("output directory does not exist");
    }

    let dir_metadata = fs::metadata(output_dir);
    match dir_metadata {
        Ok(md) => {
            if md.permissions().readonly() {
                panic!("you do not have permission to change the output directory");
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
    fn validate_args_good_file_and_directory() {
        // assign
        let file_name = PathBuf::from("testdata.txt");
        let dir = PathBuf::from(".");

        // act
        validate_args(&file_name, &dir);

        // assert
        assert!(true);
    }

    #[test]
    #[should_panic(expected = "file_name does not exist")]
    fn validate_args_file_does_not_exist() {
        // assign
        let file_name = PathBuf::from("badtestdata.txt");
        let dir = PathBuf::from(".");

        // act
        validate_args(&file_name, &dir);

        // assert
        assert!(false)
    }

    #[test]
    #[should_panic(expected = "output directory does not exist")]
    fn validate_args_directory_does_not_exist() {
        // assign
        let file_name = PathBuf::from("testdata.txt");
        let dir = PathBuf::from("baddirectory");

        // act
        validate_args(&file_name, &dir);

        // assert
        assert!(false);
    }

    #[test]
    #[should_panic(expected = "you do not have permission to change the output directory")]
    fn validate_args_directory_does_not_have_permissions() {
        // assign
        let file_name = PathBuf::from("testdata.txt");
        let dir = PathBuf::from("/dev");

        // act
        validate_args(&file_name, &dir);

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
