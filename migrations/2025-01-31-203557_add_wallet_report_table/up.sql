-- Your SQL goes here
CREATE TYPE rating_classification AS ENUM ('aaa', 'aa', 'a', 'bbb', 'bb', 'b', 'ccc', 'cc', 'c');
CREATE TABLE "wallet_report"(
	"id" UUID NOT NULL PRIMARY KEY,
	"rating_classification" rating_classification NOT NULL,
	"rating_score" INTEGER NOT NULL,
	"case_report" JSONB NOT NULL,
	"report_creation_date" TIMESTAMP NOT NULL
);
