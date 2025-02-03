-- Your SQL goes here
CREATE TABLE "users"(
	"id" UUID NOT NULL PRIMARY KEY,
	"api_key" TEXT NOT NULL,
	"created_at" TIMESTAMP NOT NULL
);
