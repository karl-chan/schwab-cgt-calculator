use chrono::NaiveDate;
use std::collections::BTreeMap;

pub trait ExchangeRateProvider {
    fn to_gbp(&self, usd: f64, date: &NaiveDate) -> Option<f64>;
}

pub struct YahooExchangeRateProvider {
    historic_rates: BTreeMap<NaiveDate, f64>,
}

impl YahooExchangeRateProvider {
    pub async fn new() -> Self {
        let mut historic_rates: BTreeMap<NaiveDate, f64> = BTreeMap::new();
        let url = "https://query1.finance.yahoo.com/v7/finance/download/USDGBP=X?period1=0&period2=9999999999&interval=1d&events=history&includeAdjustedClose=true";
        let body = reqwest::get(url).await.unwrap().text().await.unwrap();
        let mut reader = csv::Reader::from_reader(body.as_bytes());
        for result in reader.records() {
            let record = result.unwrap();
            let date = NaiveDate::parse_from_str(&record[0], "%Y-%m-%d")
                .expect(format!("Failed to parse date from string: {:?}", &record[0]).as_str());
            let close: Option<f64> = record[5].parse().ok();
            close.map(|c| historic_rates.insert(date, c));
        }
        Self { historic_rates }
    }
}

impl ExchangeRateProvider for YahooExchangeRateProvider {
    fn to_gbp(&self, usd: f64, date: &NaiveDate) -> Option<f64> {
        let maybe_rate = if self.historic_rates.contains_key(date) {
            self.historic_rates.get(date)
        } else {
            self.historic_rates
                .range(..date)
                .next_back()
                .map(|(_closest_date, rate)| rate)
        };

        maybe_rate.map(|rate| rate * usd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_to_gbp() {
        let provider = YahooExchangeRateProvider::new().await;
        assert_eq!(
            provider.to_gbp(1.0, &NaiveDate::from_ymd(2003, 12, 1)),
            Some(0.581870)
        );
        // Use data from Friday if query falls on a weekend
        assert_eq!(
            provider.to_gbp(1.0, &NaiveDate::from_ymd(2003, 12, 7)),
            Some(0.577000)
        );
        // Earliest available date is 2003-12-01
        assert_eq!(provider.to_gbp(1.0, &NaiveDate::from_ymd(1900, 1, 1)), None);
    }
}
