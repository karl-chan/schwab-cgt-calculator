use std::fmt::{Display, Result};

use crate::{
    currency::exchange_rates::{ExchangeRateProvider, YahooExchangeRateProvider},
    schwab::equity_award_center::EquityAwardCenter,
    stock::prices::{StockPriceProvider, YahooStockPriceProvider},
};
use chrono::NaiveDate;

pub struct CGTCalculatorResult {
    pub cgt: f64,
    pub proceeds: f64,
    pub cost: f64,
    pub amount_subject_to_cgt: f64,
    pub cgt_rate: f64,
}

impl Display for CGTCalculatorResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        writeln!(
            f,
            "
=============================
CGT due: £{:.2}
=============================
Breakdown:
* Proceeds: £{:.2}
* Cost: £{:.2}
* Net proceeds: £{:.2}
* Amount subject to CGT: £{:.2}
* CGT Rate: {}%
* Net proceeds: £{:.2}",
            self.cgt,
            self.proceeds,
            self.cost,
            self.proceeds - self.cost,
            self.amount_subject_to_cgt,
            self.cgt_rate * 100.0,
            self.proceeds - self.cgt
        )
    }
}

pub struct CGTCalculator {
    annual_exemption_amount: f64,
    cgt_rate: f64,
    equity_award_center: EquityAwardCenter,
    stock_price_provider: Box<dyn StockPriceProvider>,
    exchange_rate_provider: Box<dyn ExchangeRateProvider>,
}

impl CGTCalculator {
    pub async fn new(
        symbol: &str,
        equity_award_center: EquityAwardCenter,
        annual_exemption_amount: f64,
        cgt_rate: f64,
    ) -> Self {
        Self {
            annual_exemption_amount,
            cgt_rate,
            equity_award_center,
            stock_price_provider: Box::new(YahooStockPriceProvider::new(symbol).await),
            exchange_rate_provider: Box::new(YahooExchangeRateProvider::new().await),
        }
    }

    pub fn calculate_cgt(
        &self,
        symbol: &str,
        shares_to_sell: f64,
        sell_date: &NaiveDate,
    ) -> CGTCalculatorResult {
        let proceeds = self.calculate_proceeds(symbol, shares_to_sell, sell_date);
        let cost = self.calculate_cost(symbol, shares_to_sell, sell_date);
        let amount_subject_to_cgt = (proceeds - cost - self.annual_exemption_amount).max(0.0);
        let cgt = amount_subject_to_cgt * self.cgt_rate;

        CGTCalculatorResult {
            cgt,
            proceeds,
            cost,
            amount_subject_to_cgt,
            cgt_rate: self.cgt_rate,
        }
    }

    pub fn calculate_proceeds(
        &self,
        symbol: &str,
        shares_to_sell: f64,
        sell_date: &NaiveDate,
    ) -> f64 {
        let sell_price = self
            .stock_price_provider
            .get_historic_price(symbol, sell_date)
            .expect(format!("Missing stock price for date: {:?}", sell_date).as_str());

        self.exchange_rate_provider
            .to_gbp(sell_price * shares_to_sell, sell_date)
            .expect(format!("Missing exchange rate for date: {:?}", sell_date).as_str())
    }

    pub fn calculate_cost(&self, symbol: &str, shares_to_sell: f64, sell_date: &NaiveDate) -> f64 {
        let records_before_sell_date = self
            .equity_award_center
            .awards
            .iter()
            .filter(|award| award.symbol == symbol)
            .filter(|award| award.date_acquired <= sell_date.to_owned())
            .collect::<Vec<_>>();

        let total_cost_before_sell_date: f64 = records_before_sell_date
            .iter()
            .map(|award| {
                self.exchange_rate_provider
                    .to_gbp(
                        award.available_to_sell * award.acquisition_price,
                        &award.date_acquired,
                    )
                    .expect(
                        format!("Missing exchange rate for date: {:?}", &award.date_acquired)
                            .as_str(),
                    )
            })
            .sum();
        let total_shares_before_sell_date: f64 = records_before_sell_date
            .iter()
            .map(|award| award.available_to_sell)
            .sum();
        let avg_cost = total_cost_before_sell_date / total_shares_before_sell_date;

        avg_cost * shares_to_sell
    }
}
