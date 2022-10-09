use std::fmt::{Display, Result};

use crate::{
    currency::exchange_rates::{ExchangeRateProvider, YahooExchangeRateProvider},
    schwab::equity_award_center::EquityAwardCenter,
    stock::prices::{StockPriceProvider, YahooStockPriceProvider},
};
use chrono::{Duration, NaiveDate};

pub struct CGTCalculatorResult {
    pub cgt: f64,
    pub proceeds: f64,
    pub bed_and_breakfast_cost: f64,
    pub section_104_holding_cost: f64,
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
* Bed & Breakfast Cost: £{:.2}
* Section 104 Holdings Cost: £{:.2}
* Net proceeds: £{:.2}
* Amount subject to CGT: £{:.2}
* CGT Rate: {}%",
            self.cgt,
            self.proceeds,
            self.bed_and_breakfast_cost,
            self.section_104_holding_cost,
            self.proceeds - self.bed_and_breakfast_cost - self.section_104_holding_cost,
            self.amount_subject_to_cgt,
            self.cgt_rate * 100.0,
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
        self.validate_sufficient_holdings_at_sell_date(symbol, shares_to_sell, sell_date);

        let proceeds = self.calculate_proceeds(symbol, shares_to_sell, sell_date);
        let (bed_and_breakfast_cost, section_104_holding_cost) =
            self.calculate_costs(symbol, shares_to_sell, sell_date);
        let amount_subject_to_cgt = (proceeds
            - bed_and_breakfast_cost
            - section_104_holding_cost
            - self.annual_exemption_amount)
            .max(0.0);
        let cgt = amount_subject_to_cgt * self.cgt_rate;

        CGTCalculatorResult {
            cgt,
            proceeds,
            bed_and_breakfast_cost,
            section_104_holding_cost,
            amount_subject_to_cgt,
            cgt_rate: self.cgt_rate,
        }
    }

    fn validate_sufficient_holdings_at_sell_date(
        &self,
        symbol: &str,
        shares_to_sell: f64,
        sell_date: &NaiveDate,
    ) {
        let available_shares_at_sell_date: f64 = self
            .equity_award_center
            .awards
            .iter()
            .filter(|award| award.symbol == symbol)
            .filter(|award| award.date_acquired <= sell_date.to_owned())
            .map(|award| award.available_to_sell)
            .sum();

        if shares_to_sell > available_shares_at_sell_date {
            panic!(
                "You tried to sell {} {} shares, but there are only {} available before {}!",
                shares_to_sell, symbol, available_shares_at_sell_date, sell_date
            );
        }
    }

    fn calculate_proceeds(&self, symbol: &str, shares_to_sell: f64, sell_date: &NaiveDate) -> f64 {
        let sell_price = self
            .stock_price_provider
            .get_historic_price(symbol, sell_date)
            .expect(format!("Missing stock price for date: {:?}", sell_date).as_str());

        self.exchange_rate_provider
            .to_gbp(sell_price * shares_to_sell, sell_date)
            .expect(format!("Missing exchange rate for date: {:?}", sell_date).as_str())
    }

    fn calculate_costs(
        &self,
        symbol: &str,
        shares_to_sell: f64,
        sell_date: &NaiveDate,
    ) -> (f64, f64) {
        let bed_and_breakfast_lookahead_date = sell_date.to_owned() + Duration::days(30);
        let bed_and_breakfast_shares = self.count_bed_and_breakfast_shares(
            symbol,
            shares_to_sell,
            sell_date,
            &bed_and_breakfast_lookahead_date,
        );
        let section_104_holding_shares = shares_to_sell - bed_and_breakfast_shares;

        (
            self.calculate_bed_and_breakfast_cost(
                symbol,
                bed_and_breakfast_shares,
                sell_date,
                &bed_and_breakfast_lookahead_date,
            ),
            self.calculate_section_104_holding_cost(symbol, section_104_holding_shares, sell_date),
        )
    }

    fn count_bed_and_breakfast_shares(
        &self,
        symbol: &str,
        shares_to_sell: f64,
        sell_date: &NaiveDate,
        lookahead_date: &NaiveDate,
    ) -> f64 {
        let lookahead_shares: f64 = self
            .equity_award_center
            .awards
            .iter()
            .filter(|award| award.symbol == symbol)
            .filter(|award| {
                sell_date.to_owned() <= award.date_acquired.to_owned()
                    && award.date_acquired.to_owned() <= lookahead_date.to_owned()
            })
            .map(|award| award.available_to_sell)
            .sum();

        lookahead_shares.min(shares_to_sell)
    }

    fn calculate_bed_and_breakfast_cost(
        &self,
        symbol: &str,
        shares_to_sell: f64,
        sell_date: &NaiveDate,
        lookahead_date: &NaiveDate,
    ) -> f64 {
        let mut lookahead_awards = self
            .equity_award_center
            .awards
            .iter()
            .filter(|award| award.symbol == symbol)
            .filter(|award| {
                sell_date.to_owned() <= award.date_acquired.to_owned()
                    && award.date_acquired.to_owned() <= lookahead_date.to_owned()
            })
            .collect::<Vec<_>>();
        lookahead_awards.sort_by_key(|award| award.date_acquired);

        let mut total_cost = 0.0;
        let mut remaining_shares_to_fill = shares_to_sell;
        for award in lookahead_awards {
            let shares_to_fill = remaining_shares_to_fill.min(award.available_to_sell);
            total_cost += self
                .exchange_rate_provider
                .to_gbp(
                    shares_to_fill * award.acquisition_price,
                    &award.date_acquired,
                )
                .expect(
                    format!("Missing exchange rate for date: {:?}", &award.date_acquired).as_str(),
                );
            remaining_shares_to_fill -= shares_to_fill;
        }
        total_cost
    }

    fn calculate_section_104_holding_cost(
        &self,
        symbol: &str,
        shares_to_sell: f64,
        sell_date: &NaiveDate,
    ) -> f64 {
        let awards_before_sell_date = self
            .equity_award_center
            .awards
            .iter()
            .filter(|award| award.symbol == symbol)
            .filter(|award| award.date_acquired < sell_date.to_owned())
            .collect::<Vec<_>>();

        let total_cost_before_sell_date: f64 = awards_before_sell_date
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
        let total_shares_before_sell_date: f64 = awards_before_sell_date
            .iter()
            .map(|award| award.available_to_sell)
            .sum();
        let avg_cost = total_cost_before_sell_date / total_shares_before_sell_date;

        avg_cost * shares_to_sell
    }
}
