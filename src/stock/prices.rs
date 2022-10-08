use std::collections::BTreeMap;

use chrono::NaiveDate;

pub trait StockPriceProvider {
    fn get_historic_price(&self, symbol: &str, date: &NaiveDate) -> Option<f64>;
}

pub struct YahooStockPriceProvider {
    symbol: String,
    historic_prices: BTreeMap<NaiveDate, f64>,
}

impl YahooStockPriceProvider {
    pub async fn new(symbol: &str) -> Self {
        let mut historic_prices: BTreeMap<NaiveDate, f64> = BTreeMap::new();
        let url = format!("https://query1.finance.yahoo.com/v7/finance/download/{}?period1=0&period2=9999999999&interval=1d&events=history&includeAdjustedClose=true", symbol);
        let body = reqwest::get(url).await.unwrap().text().await.unwrap();
        let mut reader = csv::Reader::from_reader(body.as_bytes());
        for result in reader.records() {
            let record = result.unwrap();
            let date = NaiveDate::parse_from_str(&record[0], "%Y-%m-%d")
                .expect(format!("Failed to parse date from string: {:?}", &record[0]).as_str());
            let close: Option<f64> = record[5].parse().ok();
            close.map(|c| historic_prices.insert(date, c));
        }
        Self {
            symbol: symbol.to_owned(),
            historic_prices,
        }
    }
}

impl StockPriceProvider for YahooStockPriceProvider {
    fn get_historic_price(&self, symbol: &str, date: &NaiveDate) -> Option<f64> {
        assert_eq!(
            symbol, self.symbol,
            "This stock price provider only supports {}, not {}!",
            self.symbol, symbol
        );

        let maybe_price = if self.historic_prices.contains_key(date) {
            self.historic_prices.get(date)
        } else {
            self.historic_prices
                .range(..date)
                .next_back()
                .map(|(_closest_date, price)| price)
        };

        maybe_price.map(|price| price.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_historic_price() {
        let provider = YahooStockPriceProvider::new("GOOG").await;
        assert_eq!(
            provider.get_historic_price("GOOG", &NaiveDate::from_ymd(2004, 08, 19)),
            Some(2.499133)
        );
        // Earliest available date is 2004-08-19
        assert_eq!(
            provider.get_historic_price("GOOG", &NaiveDate::from_ymd(1900, 1, 1)),
            None
        );
    }
}
