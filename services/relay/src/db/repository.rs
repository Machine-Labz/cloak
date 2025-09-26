use async_trait::async_trait;
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
    async fn update_job_completed(&self, id: Uuid, tx_id: String, signature: String) -> Result<(), Error>;
    async fn update_job_failed(&self, id: Uuid, error: String) -> Result<(), Error>;
    async fn increment_retry_count(&self, id: Uuid) -> Result<(), Error>;
    async fn get_queued_jobs(&self, limit: i64) -> Result<Vec<Job>, Error>;
    async fn get_jobs_by_status(&self, status: JobStatus) -> Result<Vec<JobSummary>, Error>;
}

#[async_trait]
pub trait NullifierRepository: Send + Sync {
    async fn create_nullifier(&self, nullifier: Vec<u8>, job_id: Uuid) -> Result<(), Error>;
    async fn exists_nullifier(&self, nullifier: &[u8]) -> Result<bool, Error>;
    async fn get_nullifier(&self, nullifier: &[u8]) -> Result<Option<Nullifier>, Error>;
    async fn update_nullifier_block_info(&self, nullifier: &[u8], block_height: i64, tx_signature: String) -> Result<(), Error>;
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
        let created_job = sqlx::query_as!(
            Job,
            r#"
            INSERT INTO jobs (
                request_id, proof_bytes, public_inputs, outputs_json, fee_bps,
                root_hash, nullifier, amount, outputs_hash
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING
                id, request_id, status as "status: JobStatus", proof_bytes, public_inputs, 
                outputs_json, fee_bps, root_hash, nullifier, amount, outputs_hash,
                tx_id, solana_signature, error_message, retry_count, max_retries,
                created_at, updated_at, started_at, completed_at
            "#,
            job.request_id,
            job.proof_bytes,
            job.public_inputs,
            job.outputs_json,
            job.fee_bps,
            job.root_hash,
            job.nullifier,
            job.amount,
            job.outputs_hash
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to create job: {}", e)))?;

        Ok(created_job)
    }

    async fn get_job_by_id(&self, id: Uuid) -> Result<Option<Job>, Error> {
        let job = sqlx::query_as!(
            Job,
            r#"
            SELECT
                id, request_id, status as "status: JobStatus", proof_bytes, public_inputs,
                outputs_json, fee_bps, root_hash, nullifier, amount, outputs_hash,
                tx_id, solana_signature, error_message, retry_count, max_retries,
                created_at, updated_at, started_at, completed_at
            FROM jobs WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to get job by id: {}", e)))?;

        Ok(job)
    }

    async fn get_job_by_request_id(&self, request_id: Uuid) -> Result<Option<Job>, Error> {
        let job = sqlx::query_as!(
            Job,
            r#"
            SELECT
                id, request_id, status as "status: JobStatus", proof_bytes, public_inputs,
                outputs_json, fee_bps, root_hash, nullifier, amount, outputs_hash,
                tx_id, solana_signature, error_message, retry_count, max_retries,
                created_at, updated_at, started_at, completed_at
            FROM jobs WHERE request_id = $1
            "#,
            request_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to get job by request_id: {}", e)))?;

        Ok(job)
    }

    async fn update_job_status(&self, id: Uuid, status: JobStatus) -> Result<(), Error> {
        sqlx::query!(
            r#"UPDATE jobs SET status = $1 WHERE id = $2"#,
            status as JobStatus,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to update job status: {}", e)))?;

        Ok(())
    }

    async fn update_job_processing(&self, id: Uuid, tx_id: Option<String>) -> Result<(), Error> {
        sqlx::query!(
            r#"
            UPDATE jobs 
            SET status = 'processing', started_at = NOW(), tx_id = $1
            WHERE id = $2
            "#,
            tx_id,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to update job processing: {}", e)))?;

        Ok(())
    }

    async fn update_job_completed(&self, id: Uuid, tx_id: String, signature: String) -> Result<(), Error> {
        sqlx::query!(
            r#"
            UPDATE jobs 
            SET status = 'completed', completed_at = NOW(), tx_id = $1, solana_signature = $2
            WHERE id = $3
            "#,
            tx_id,
            signature,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to update job completed: {}", e)))?;

        Ok(())
    }

    async fn update_job_failed(&self, id: Uuid, error: String) -> Result<(), Error> {
        sqlx::query!(
            r#"
            UPDATE jobs 
            SET status = 'failed', completed_at = NOW(), error_message = $1
            WHERE id = $2
            "#,
            error,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to update job failed: {}", e)))?;

        Ok(())
    }

    async fn increment_retry_count(&self, id: Uuid) -> Result<(), Error> {
        sqlx::query!(
            r#"UPDATE jobs SET retry_count = retry_count + 1 WHERE id = $1"#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to increment retry count: {}", e)))?;

        Ok(())
    }

    async fn get_queued_jobs(&self, limit: i64) -> Result<Vec<Job>, Error> {
        let jobs = sqlx::query_as!(
            Job,
            r#"
            SELECT
                id, request_id, status as "status: JobStatus", proof_bytes, public_inputs,
                outputs_json, fee_bps, root_hash, nullifier, amount, outputs_hash,
                tx_id, solana_signature, error_message, retry_count, max_retries,
                created_at, updated_at, started_at, completed_at
            FROM jobs 
            WHERE status = 'queued' AND retry_count < max_retries
            ORDER BY created_at ASC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to get queued jobs: {}", e)))?;

        Ok(jobs)
    }

    async fn get_jobs_by_status(&self, status: JobStatus) -> Result<Vec<JobSummary>, Error> {
        let jobs = sqlx::query!(
            r#"
            SELECT request_id, status as "status: JobStatus", tx_id, error_message, created_at, completed_at
            FROM jobs 
            WHERE status = $1
            ORDER BY created_at DESC
            "#,
            status as JobStatus
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to get jobs by status: {}", e)))?;

        let summaries = jobs
            .into_iter()
            .map(|row| JobSummary {
                request_id: row.request_id,
                status: row.status,
                tx_id: row.tx_id,
                error_message: row.error_message,
                created_at: row.created_at,
                completed_at: row.completed_at,
            })
            .collect();

        Ok(summaries)
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
        sqlx::query!(
            r#"INSERT INTO nullifiers (nullifier, job_id) VALUES ($1, $2)"#,
            nullifier,
            job_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to create nullifier: {}", e)))?;

        Ok(())
    }

    async fn exists_nullifier(&self, nullifier: &[u8]) -> Result<bool, Error> {
        let count = sqlx::query!(
            r#"SELECT COUNT(*) as count FROM nullifiers WHERE nullifier = $1"#,
            nullifier
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to check nullifier existence: {}", e)))?;

        Ok(count.count.unwrap_or(0) > 0)
    }

    async fn get_nullifier(&self, nullifier: &[u8]) -> Result<Option<Nullifier>, Error> {
        let nullifier_record = sqlx::query_as!(
            Nullifier,
            r#"
            SELECT nullifier, job_id, block_height, tx_signature, created_at
            FROM nullifiers WHERE nullifier = $1
            "#,
            nullifier
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to get nullifier: {}", e)))?;

        Ok(nullifier_record)
    }

    async fn update_nullifier_block_info(&self, nullifier: &[u8], block_height: i64, tx_signature: String) -> Result<(), Error> {
        sqlx::query!(
            r#"
            UPDATE nullifiers 
            SET block_height = $1, tx_signature = $2
            WHERE nullifier = $3
            "#,
            block_height,
            tx_signature,
            nullifier
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to update nullifier block info: {}", e)))?;

        Ok(())
    }
} 