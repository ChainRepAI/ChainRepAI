 # ChainRepAI

 <!-- Put Image here, perhaps example -->

## Description
ChainRepAI is your personal on-chain Solana wallet analyst. Using various indicators, it's able to understand the reputation rating of the provided wallet.

This project is, and always will be, Open-Source and 100% free to use.

## Breakdown of the Reputational Indicators

### Wallet Balance
Although this indicator doesn't inherently indicate reputation, when combined with others it does illustrate the wallet holder is invested in the Solana ecosystem, especially when the wallet balance is maintained for a long period of time.

As expected, a higher wallet balances are perceived with higher reputation, and as such, very low balances are penalized.

### Transaction Volume
Regular activity over a long period of time indicates the wallet is legitimate and therefore more reputable. Low transaction volume doesn't inherently impl lower reputation although it's marginally penalized. Wallets with extremely high transaction volume may indicate the wallet is operated by a bot. As such, these wallets are heavily penalized.

To receive no or a low penalty on this indicator, wallets should maintain a reasonable level of transaction volume over an extended period of time. 

### Dormancy
Dormancy, or time since last transaction, is a considerable indicator into a wallet's reputation. Large periods of time since a transaction indicates the wallet is not frequently used.

### Transaction failure rate
High transaction failure rate may imply lower wallet reputation in several ways:
- Poor understanding of the network (insufficient gas fees, improper transaction construction)
- Attempts to front-run or engage in high-frequency activities without proper optimization
- Interaction with poorly written smart contracts

## Planned Features


## How To Run Locally


## Contributing

Before integrating a new feature, please quickly reach out to us in an issue so we can discuss and coordinate the change.

- If you find any bugs, submit an [issue](../../issues) or open [pull-request](../../pulls).


