use anyhow::Result;
use chrono::NaiveDate;

#[derive(Debug)]
pub struct EquityAwardCenter {
    pub awards: Vec<EquityAward>,
}

#[derive(Debug, PartialEq)]
pub struct EquityAward {
    pub symbol: String,
    pub date_acquired: NaiveDate,
    pub acquisition_price: f64,
    pub available_to_sell: f64,
}

impl EquityAwardCenter {
    /// Parses the EquityAwardsCenter_EquityDetails csv file.
    ///
    /// The file can be downloaded from https://client.schwab.com/app/accounts/equityawards/#/equityTodayView > Export.
    pub fn parse_from_csv(path_to_csv: &str) -> Result<Self> {
        let mut awards: Vec<EquityAward> = Vec::new();
        let mut cursor_in_equity_award_shares = false;

        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .has_headers(false)
            .from_path(path_to_csv)?;
        for result in reader.records() {
            let record = result?;
            if record.get(0) == Some("*** EQUITY AWARD SHARES ***") {
                // Start scanner
                cursor_in_equity_award_shares = true;
            }

            if cursor_in_equity_award_shares {
                // Stop scanner
                if record.get(0) == Some("Totals") {
                    cursor_in_equity_award_shares = false;
                }

                // Verify columns
                if record.get(0) == Some("Award Date") {
                    assert_eq!(
                        record.into_iter().collect::<Vec<_>>(),
                        vec![
                            "Award Date",
                            "Symbol",
                            "Award ID",
                            "Share Type",
                            "Market Value",
                            "Date Holding Period Met",
                            "Deposit Date",
                            "Date Acquired",
                            "Acquisition Price",
                            "Shares",
                            "Available to Sell",
                        ],
                    );
                }

                // Try parse record
                let is_record = record
                    .get(0)
                    .map(|s| NaiveDate::parse_from_str(s, "%m-%d-%Y").is_ok())
                    .unwrap_or(false);
                if is_record {
                    let symbol = record[1].to_owned();
                    let date_acquired = NaiveDate::parse_from_str(&record[7], "%m-%d-%Y").unwrap();
                    let acquisition_price: f64 = record[8].replace("$", "").parse().unwrap();
                    let available_to_sell: f64 = record[10].parse().unwrap();

                    let award = EquityAward {
                        symbol,
                        date_acquired,
                        acquisition_price,
                        available_to_sell,
                    };
                    awards.push(award);
                }
            }
        }

        Ok(Self { awards })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let equity_award_center = EquityAwardCenter::parse_from_csv(
            "assets/private/EquityAwardsCenter_EquityDetails.csv",
        )
        .unwrap();
        assert_eq!(equity_award_center.awards.len(), 31);
        assert_eq!(
            equity_award_center.awards.first().unwrap(),
            &EquityAward {
                symbol: String::from("GOOG"),
                date_acquired: NaiveDate::from_ymd(2018, 11, 26),
                acquisition_price: 51.194,
                available_to_sell: 42.42
            }
        );
        assert_eq!(
            equity_award_center.awards.last().unwrap(),
            &EquityAward {
                symbol: String::from("GOOG"),
                date_acquired: NaiveDate::from_ymd(2022, 9, 25),
                acquisition_price: 99.17,
                available_to_sell: 10.351
            }
        );
    }
}
