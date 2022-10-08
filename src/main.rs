use chrono::NaiveDate;
use clap::{Parser, ValueEnum};
use schwab_cgt_calculator::{
    calculator::CGTCalculator, schwab::equity_award_center::EquityAwardCenter,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Stock symbol
    #[arg(long)]
    symbol: String,

    /// Sell date (YYYY-MM-DD)
    #[arg(long)]
    sell_date: NaiveDate,

    /// Number of shares to sell
    #[arg(long)]
    shares_to_sell: f64,

    /// Path to EquityAwardsCenter_EquityDetails_yyyymmxxxxxx.csv file.
    #[arg(long)]
    path_to_csv: String,

    /// Annual exemption amount (£12,300 for 2022)
    #[arg(long, default_value_t = 12300.0)]
    annual_exemption_amount: f64,

    ///  Taxpayer status (Basic rate - 10% / Higher rate - 20%)
    #[arg(long, value_enum)]
    taxpayer_status: TaxpayerStatus,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum TaxpayerStatus {
    Basic,
    Higher,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();

    let cgt_rate = match args.taxpayer_status {
        TaxpayerStatus::Basic => 0.1,
        TaxpayerStatus::Higher => 0.2,
    };

    let calculator = CGTCalculator::new(
        &args.symbol,
        EquityAwardCenter::parse_from_csv(&args.path_to_csv).unwrap(),
        args.annual_exemption_amount,
        cgt_rate,
    )
    .await;
    let cgt_result = calculator.calculate_cgt(&args.symbol, args.shares_to_sell, &args.sell_date);

    println!("{}", cgt_result.to_string())
}
