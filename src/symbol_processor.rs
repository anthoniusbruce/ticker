pub mod symbol_processor {
    use std::{fs::File, path::PathBuf};

    use csv::Writer;
    use tokio::runtime::Builder;
    use yahoo_finance_api::{time::OffsetDateTime, Quote};

    use crate::log;

    /// using the list of symbols get the daily quotes for the past month
    pub fn process_symbols(symbols: Vec<&str>, output_dir: &PathBuf) {
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
            Err(e) => log(symbol, e),
            Ok(file) => {
                let mut writer = Writer::from_writer(file);
                let serialize_result = writer.serialize(gains);
                match serialize_result {
                    Err(e) => log(symbol, e),
                    Ok(_) => (),
                }
                let flush_result = writer.flush();
                match flush_result {
                    Err(e) => log(symbol, e),
                    Ok(_) => (),
                }
            }
        }
    }

    /// converts Quote to the single value of the gain of the day (+/-)
    pub(crate) fn get_gain(quote: Quote) -> f64 {
        quote.close - quote.open
    }

    /// Method to get that quotes over a duration for a given ticker symbol
    fn get_quotes(symbol: &str, start: &OffsetDateTime, end: &OffsetDateTime) -> Vec<Quote> {
        let quotes = Vec::new();
        let provider_result = yahoo_finance_api::YahooConnector::new();
        match provider_result {
            Err(e) => log(symbol, e),
            Ok(provider) => {
                let builder_result = Builder::new_current_thread().enable_all().build();
                match builder_result {
                    Err(e) => log(symbol, e),
                    Ok(builder) => {
                        let resp_result =
                            builder.block_on(provider.get_quote_history(symbol, *start, *end));
                        match resp_result {
                            Err(e) => log(symbol, e),
                            Ok(resp) => match resp.quotes() {
                                Err(e) => log(symbol, e),
                                Ok(response) => {
                                    let message =
                                        format!("Success: {} - {}", start.date(), end.date(),);
                                    log(symbol, message);
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
}