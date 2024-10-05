/// Tests
#[cfg(test)]
pub mod unit_tests {
    use std::{fs, path::PathBuf};

    use yahoo_finance_api::Quote;

    use crate::symbol_processor::symbol_processor::get_gain;
    use crate::{get_ticker_symbols, read_file, validate_args};

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
