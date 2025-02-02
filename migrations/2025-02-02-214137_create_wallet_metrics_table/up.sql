-- Your SQL goes here
CREATE TABLE "wallet_metrics"(
	"wallet_report_id" UUID NOT NULL PRIMARY KEY,
	"transaction_failure_rate" DOUBLE PRECISION NOT NULL,
	"avg_prio_fee" DOUBLE PRECISION NOT NULL,
	"prio_fee_std_devi" DOUBLE PRECISION NOT NULL,
	"days_since_last_block" BIGINT NOT NULL,
	"tx_per_hour" BIGINT NOT NULL,
	"wallet_balance" BIGINT NOT NULL
);
