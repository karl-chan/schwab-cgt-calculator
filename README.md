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
* Cost: £91539.52
* Net proceeds: £13991.93
* Amount subject to CGT: £1691.93
* CGT Rate: 20%
* Net proceeds: £105193.06
```

## Done
- [X] Section 104 holding (https://www.gov.uk/tax-sell-shares/same-company)
- [X] Exchange rate conversion

## TODO
- [ ] Same day sale
- [ ] Bed & breakfast
- [ ] Carry losses forwards
- [ ] Verify there are sufficient shares at sell date
