# Ticker

**Ticker** is a command-line application written in Rust that retrieves historical stock ticker data from Yahoo Finance. The application reads ticker symbols from a file, fetches the last month of data for each ticker, and outputs the daily gains or losses for each ticker symbol into a specified output directory.

## Features

- Fetches historical stock data from Yahoo Finance using the `yahoo_finance` module.
- Reads ticker symbols from a provided file.
- Outputs daily stock gains or losses into individual files named after the ticker symbols.
- Supports logging with a customizable log file.

## Usage

```bash
ticker <file-name> <output> <log-file>
```

## Example
ticker tickers.csv output/ ticker.log
In this example:
- tickers.csv is the input file containing ticker symbols.
- output/ is the directory where the daily stock information for each ticker will be written.
- ticker.log is the log file that records application logs and any errors encountered during execution.

## Input File Format
The input file should be a CSV file with one ticker symbols. Example
```
AAPL,GOOGL,MSFT
```
## Output
For each ticker symbol in the input file, a corresponding file will be created in the output directory (<output>). Each file will be named after the ticker symbol (e.g., AAPL, GOOGL, MSFT) and will contain daily gains or losses for the past month.

## Log File
All logs, including any errors encountered, will be written to the specified <log-file>.

## Installation

Ensure you have Rust installed. Then, clone this repository and run the following command to build the application:
```bash
cargo build --release
```
You can then run the executable from the target/release directory.

## License
This project is licensed under the MIT License.
