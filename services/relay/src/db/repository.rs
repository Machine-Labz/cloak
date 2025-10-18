use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

use super::models::{CreateJob, Job, JobStatus, JobSummary, Nullifier};
use super::DatabasePool;
use crate::error::Error;

#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn create_job(&self, job: CreateJob) -> Result<Job, Error>;
    async fn get_job_by_id(&self, id: Uuid) -> Result<Option<Job>, Error>;
    async fn get_job_by_request_id(&self, request_id: Uuid) -> Result<Option<Job>, Error>;
    async fn update_job_status(&self, id: Uuid, status: JobStatus) -> Result<(), Error>;
    async fn update_job_processing(&self, id: Uuid, tx_id: Option<String>) -> Result<(), Error>;
    async fn update_job_completed(
        &self,
        id: Uuid,
        tx_id: String,
        signature: String,
    ) -> Result<(), Error>;
    async fn update_job_failed(&self, id: Uuid, error: String) -> Result<(), Error>;
    async fn update_job_proof(&self, id: Uuid, proof_bytes: Vec<u8>, public_inputs: Vec<u8>) -> Result<(), Error>;
    async fn increment_retry_count(&self, id: Uuid) -> Result<(), Error>;
    async fn get_queued_jobs(&self, limit: i64) -> Result<Vec<Job>, Error>;
    async fn get_jobs_by_status(&self, status: JobStatus) -> Result<Vec<JobSummary>, Error>;
}

#[async_trait]
pub trait NullifierRepository: Send + Sync {
    async fn create_nullifier(&self, nullifier: Vec<u8>, job_id: Uuid) -> Result<(), Error>;
    async fn exists_nullifier(&self, nullifier: &[u8]) -> Result<bool, Error>;
    async fn get_nullifier(&self, nullifier: &[u8]) -> Result<Option<Nullifier>, Error>;
    async fn update_nullifier_block_info(
        &self,
        nullifier: &[u8],
        block_height: i64,
        tx_signature: String,
    ) -> Result<(), Error>;
}

pub struct PostgresJobRepository {
    pool: DatabasePool,
}

impl PostgresJobRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JobRepository for PostgresJobRepository {
    async fn create_job(&self, job: CreateJob) -> Result<Job, Error> {
        let created_job = sqlx::query_as::<_, Job>(
            "INSERT INTO jobs (request_id, proof_bytes, public_inputs, outputs_json, fee_bps, root_hash, nullifier, amount, outputs_hash) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id, request_id, status, proof_bytes, public_inputs, outputs_json, fee_bps, root_hash, nullifier, amount, outputs_hash, tx_id, solana_signature, error_message, retry_count, max_retries, created_at, updated_at, started_at, completed_at"
        )
            .bind(job.request_id)
            .bind(job.proof_bytes)
            .bind(job.public_inputs)
            .bind(job.outputs_json)
            .bind(job.fee_bps)
            .bind(job.root_hash)
            .bind(job.nullifier)
            .bind(job.amount)
            .bind(job.outputs_hash)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(format!("Failed to create job: {}", e)))?;

        Ok(created_job)
    }

    async fn get_job_by_id(&self, id: Uuid) -> Result<Option<Job>, Error> {
        let job = sqlx::query_as::<_, Job>(
            "SELECT
                id, request_id, status, proof_bytes, public_inputs,
                outputs_json, fee_bps, root_hash, nullifier, amount, outputs_hash,
                tx_id, solana_signature, error_message, retry_count, max_retries,
                created_at, updated_at, started_at, completed_at
            FROM jobs WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to get job by id: {}", e)))?;

        Ok(job)
    }

    async fn get_job_by_request_id(&self, request_id: Uuid) -> Result<Option<Job>, Error> {
        let job = sqlx::query_as::<_, Job>(
            "SELECT
                id, request_id, status, proof_bytes, public_inputs,
                outputs_json, fee_bps, root_hash, nullifier, amount, outputs_hash,
                tx_id, solana_signature, error_message, retry_count, max_retries,
                created_at, updated_at, started_at, completed_at
            FROM jobs WHERE request_id = $1",
        )
        .bind(request_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to get job by request_id: {}", e)))?;

        Ok(job)
    }

    async fn update_job_status(&self, id: Uuid, status: JobStatus) -> Result<(), Error> {
        sqlx::query("UPDATE jobs SET status = $1 WHERE id = $2")
            .bind(status)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(format!("Failed to update job status: {}", e)))?;

        Ok(())
    }

    async fn update_job_processing(&self, id: Uuid, tx_id: Option<String>) -> Result<(), Error> {
        sqlx::query(
            "UPDATE jobs SET status = 'processing', started_at = NOW(), tx_id = $1 WHERE id = $2",
        )
        .bind(tx_id)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to update job processing: {}", e)))?;

        Ok(())
    }

    async fn update_job_completed(
        &self,
        id: Uuid,
        tx_id: String,
        signature: String,
    ) -> Result<(), Error> {
        sqlx::query(
            "UPDATE jobs SET status = 'completed', completed_at = NOW(), tx_id = $1, solana_signature = $2 WHERE id = $3"
        )
        .bind(tx_id)
        .bind(signature)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to update job completed: {}", e)))?;

        Ok(())
    }

    async fn update_job_failed(&self, id: Uuid, error: String) -> Result<(), Error> {
        sqlx::query(
            "UPDATE jobs SET status = 'failed', completed_at = NOW(), error_message = $1 WHERE id = $2"
        )
        .bind(error)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to update job failed: {}", e)))?;

        Ok(())
    }

    async fn update_job_proof(&self, id: Uuid, proof_bytes: Vec<u8>, public_inputs: Vec<u8>) -> Result<(), Error> {
        sqlx::query(
            "UPDATE jobs SET proof_bytes = $1, public_inputs = $2, updated_at = NOW() WHERE id = $3",
        )
        .bind(proof_bytes)
        .bind(public_inputs)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to update job proof: {}", e)))?;
        Ok(())
    }

    async fn increment_retry_count(&self, id: Uuid) -> Result<(), Error> {
        sqlx::query("UPDATE jobs SET retry_count = retry_count + 1 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(format!("Failed to increment retry count: {}", e)))?;

        Ok(())
    }

    async fn get_queued_jobs(&self, limit: i64) -> Result<Vec<Job>, Error> {
        let jobs = sqlx::query_as::<_, Job>(
            "SELECT
                id, request_id, status, proof_bytes, public_inputs,
                outputs_json, fee_bps, root_hash, nullifier, amount, outputs_hash,
                tx_id, solana_signature, error_message, retry_count, max_retries,
                created_at, updated_at, started_at, completed_at
            FROM jobs 
            WHERE status = 'queued' AND retry_count < max_retries ORDER BY created_at ASC LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to get queued jobs: {}", e)))?;

        Ok(jobs)
    }

    async fn get_jobs_by_status(&self, status: JobStatus) -> Result<Vec<JobSummary>, Error> {
        sqlx::query_as::<_, JobSummary>(
            "SELECT request_id, status, tx_id, error_message, created_at, completed_at FROM jobs WHERE status = $1 ORDER BY created_at DESC",
        )
        .bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to get jobs by status: {}", e)))
    }
}

pub struct PostgresNullifierRepository {
    pool: DatabasePool,
}

impl PostgresNullifierRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl NullifierRepository for PostgresNullifierRepository {
    async fn create_nullifier(&self, nullifier: Vec<u8>, job_id: Uuid) -> Result<(), Error> {
        sqlx::query("INSERT INTO nullifiers (nullifier, job_id) VALUES ($1, $2)")
            .bind(nullifier)
            .bind(job_id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(format!("Failed to create nullifier: {}", e)))?;

        Ok(())
    }

    async fn exists_nullifier(&self, nullifier: &[u8]) -> Result<bool, Error> {
        let count = sqlx::query("SELECT COUNT(*) FROM nullifiers WHERE nullifier = $1")
            .bind(nullifier)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                Error::DatabaseError(format!("Failed to check nullifier existence: {}", e))
            })
            .map(|row| row.get::<i64, _>("count") > 0)?;

        Ok(count)
    }

    async fn get_nullifier(&self, nullifier: &[u8]) -> Result<Option<Nullifier>, Error> {
        let nullifier_record = sqlx::query_as::<_, Nullifier>(
            "SELECT nullifier, job_id, block_height, tx_signature, created_at FROM nullifiers WHERE nullifier = $1"
        )
        .bind(nullifier)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to get nullifier: {}", e)))?;

        Ok(nullifier_record)
    }

    async fn update_nullifier_block_info(
        &self,
        nullifier: &[u8],
        block_height: i64,
        tx_signature: String,
    ) -> Result<(), Error> {
        sqlx::query(
            "UPDATE nullifiers SET block_height = $1, tx_signature = $2 WHERE nullifier = $3",
        )
        .bind(block_height)
        .bind(tx_signature)
        .bind(nullifier)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            Error::DatabaseError(format!("Failed to update nullifier block info: {}", e))
        })?;

        Ok(())
    }
}
