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

## Example
ticker tickers.csv output/ ticker.log
In this example:
- tickers.csv is the input file containing ticker symbols.
- output/ is the directory where the daily stock information for each ticker will be written.
- ticker.log is the log file that records application logs and any errors encountered during execution.

