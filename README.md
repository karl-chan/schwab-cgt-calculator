# Schwab CGT Calculator

## Usage

```
cargo run -- --symbol <stock symbol> --sell-date <YYYY-MM-DD> --shares-to-sell <number> --path-to-csv <path> --taxpayer-status [basic|higher]
```

The CSV can be downloaded from https://client.schwab.com/app/accounts/equityawards/#/equityTodayView `> Export`.

## Sample output

```
=============================
CGT due: £338.39
=============================
Breakdown:
* Proceeds: £105531.44
* Bed & Breakfast Cost: £0.00
* Section 104 Holdings Cost: £91539.52
* Net proceeds: £13991.93
* Amount subject to CGT: £1691.93
* CGT Rate: 20%
```

## Done
- [X] Bed & breakfast rule (https://www.gov.uk/government/publications/shares-and-capital-gains-tax-hs284-self-assessment-helpsheet/hs284-shares-and-capital-gains-tax-2019)
- [X] Section 104 holding (https://www.gov.uk/tax-sell-shares/same-company)
- [X] Exchange rate conversion
- [X] Verify there are sufficient shares at sell date

## TODO
- [ ] Same day sale
- [ ] Carry losses forwards
