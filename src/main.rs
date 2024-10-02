fn main() {
    ()
}

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
    fn get_ticker_symbols_11_items() {
        // assign
        let test_data = read_test_data();
        let expected = vec![
            "AACG", "AADI", "AADR", "AAL", "AAME", "AAOI", "AAON", "AAPB", "AAPD", "AAPL",
        ];

        // act
        let actual = get_ticker_symbols(&test_data);

        println!("expected: {:?}\nactual: {:?}", expected, actual);

        // assert
        assert!(vectors_are_equal(expected, actual), "vectors are not equal");
    }
}
