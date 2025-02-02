![Static Badge](https://img.shields.io/badge/Built%20With-Rust-green) ![GitHub top language](https://img.shields.io/github/languages/top/SolAnalystAI/SolAnalystAI)
![GitHub commit activity](https://img.shields.io/github/commit-activity/w/SolAnalystAI/SolAnalystAI) ![GitHub Issues or Pull Requests](https://img.shields.io/github/issues/SolAnalystAI/SolAnalystAI)
 ![GitHub License](https://img.shields.io/github/license/SolAnalystAI/SolAnalystAI) ![X (formerly Twitter) Follow](https://img.shields.io/twitter/follow/SolAnalystAI) 




 # SolAnalystAI

 ![image](images/banner-header.png)

## Description
SolAnalystAI is your personal on-chain Solana wallet analyst. Using various indicators, it's able to understand the reputation rating of the provided wallet.

This project is, and always will be, Open-Source and 100% free to use.

### Socials
[Website](https://solanalystai.com/)
<br>
[Twitter](https://x.com/SolAnalystAI)
<br>
[YouTube](https://www.youtube.com/@SolAnalystAI)

## Example Generated Report: @frankdegods
<img src="images/frank.png" width="300">

#### Wallet Address:
CRVidEDtEUTYZisCxBZkpELzhQc9eauMLR3FWg74tReL

#### Rating Classification: 
AAA

#### Rating Score: 
900

#### Report Summary:
The Solana wallet boasts an excellent rating classification of AAA with a score of 900, primarily due to its strong transaction volume and substantial balance. The absence of severe penalties coupled with a reasonable transaction failure rate reinforces its reputation. However, recent activity poses slight concerns that could influence future assessments. Overall, the wallet's attributes warrant a high degree of trustworthiness and credibility.

#### Reputation Strengths:
The wallet showcases a healthy transaction volume, averaging 14 transactions per hour. This demonstrates an active engagement with the network without overwhelming transaction spikes indicative of market manipulation or illegal activities. The balance of 4173 Solana signifies a strong financial footing, demonstrating that the wallet is not only active but is also well capitalized. Furthermore, the low transaction failure rate at 4.2% reflects efficient transaction handling, instilling confidence among its users and reducing the perceived operational risks. Together, these strengths substantiate the wallet's AAA rating, highlighting its secure and responsible handling of financial activities.

#### Reputation Challenges:
Although the overall rating is AAA, the recent activity pattern may present a risk. Frequent and unsubstantiated transactions could lead to an increased scrutiny from the ecosystem, prompting questions about the wallet’s trustworthiness. With only a 4.2% failure rate, any increase could signify potential issues, and a higher failure rate could lead to a re-evaluation of the wallet’s reliability.

#### Potential Downgrade Factors:
The wallet recently engaged in activity less than a week ago, with the last transaction occurring 2 days ago. This raises a flag, as it indicates a potential shift in user behavior or a sudden increase in activity that may lack historical consistency. If continued activity leads to more frequent transactions, it could invoke concerns over the wallet’s legitimacy. The relatively low transaction failure rate is noteworthy, but any spikes in failures could jeopardize the wallet's strong reputation.

#### Penalty Breakdown:
The absence of severe penalties indicates the wallet operates within reasonable transaction volumes, which helps maintain a stable reputation. With an average of 14 transactions per hour, this wallet shows a consistent yet moderate level of activity, reducing risks of sudden or suspicious behavior that might trigger scrutiny. Additionally, the wallet's balance of 4173 Solana significantly exceeds the threshold of 100 Solana, further bolstering its credibility. Low transaction failure rates at 4.2% also suggest careful management of transactions, enhancing user confidence and minimizing exposure to risks associated with technical issues.

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

### Prioritization Fee Metrics
Two metrics form this indicator: average fee and fee standard deviation. Lower standard deviation implies higher wallet reputation as it reflects higher stability towards transaction management and a consistent strategy. Higher average prioritization fees also imply higher wallet reputation as it can imply the wallet is more confident in their transactions and take active approaches to mitigate front-running attempts.

### Wallet Balance Volatility
Significant fluctuations in wallet balances, characterized by high volatility and large swings in available capital, are often perceived as a red flag. Such behavior suggests that the wallet owner may not be actively managing their risk exposure, potentially exposing themselves to unnecessary financial instability. Additionally, erratic balance changes can indicate that the wallet is being used primarily as a "hot wallet," where funds are only temporarily stored to facilitate transactions rather than representing a long-term holding or actively managed portfolio. This transient nature can raise concerns about the wallet’s reliability and the intentions behind its usage.

Conversely, wallets that exhibit low balance volatility, coupled with consistent transaction activity and a reasonable, stable volume of trades, are generally regarded as highly trustworthy. This pattern indicates a well-managed account with a responsible approach to risk, reinforcing confidence in its legitimacy. Such wallets are typically associated with reputable market participants, long-term investors, or institutional users, making them strong indicators of reliability and high reputation within the Solana ecosystem.

### Percentage of transactions to new wallets
A high volume of transactions directed toward newly created Solana wallets can indicate low reputation and trustworthiness, as it may suggest artificial or suspicious trading activity. These transactions could be linked to potential wash trading, bot-driven market manipulation, or other forms of fraudulent behavior, making them less reliable indicators of legitimate market participation.

On the other hand, transactions involving well-established wallets—those with a longer history of activity, a substantial Solana balance, and a pattern of consistent, reliable transactions—are generally viewed as more trustworthy. These wallets are often associated with experienced traders, institutional participants, or long-term investors, making them less likely to be involved in deceptive practices. As a result, they tend to inspire greater confidence within the ecosystem and are considered stronger indicators of authentic trading behavior.

## How to run locally
1. Clone down the project
    ```console
    git clone git@github.com:SolAnalystAI SolAnalystAI.git
    ```
2. Create a .env file and define the following environment variables:
    ```
    RPC_URL = "https://api.mainnet-beta.solana.com"
    OPENAI_API_KEY="<redacted>"
    CASE_REPORT_PROMPT="<redacted>"
    DATABASE_URL="postgres://localhost/db_name"
    ```

3. Postgres running with the required database created (name specified in the .env file)
4. Pulsar running
5. In separate terminals, run the following commands to start the binaries:
    ```console
    cargo run --bin api_web_server
    cargo run --bin report_worker
    ```

## Contributing

Before integrating a new feature, please quickly reach out to us in an issue so we can discuss and coordinate the change.

- If you find any bugs, submit an [issue](../../issues) or open [pull-request](../../pulls).


