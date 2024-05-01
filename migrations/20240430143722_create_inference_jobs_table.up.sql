-- Add up migration script here
CREATE TYPE job_status AS ENUM ('bot', 'human', 'success', 'fail');

CREATE TABLE IF NOT EXISTS inference_jobs
(
	job_id     TEXT,
	status     job_status               NOT NULL DEFAULT 'bot',
	payload    bytea                    NOT NULL,
	response   bytea,
	created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
	PRIMARY KEY ( job_id )
);