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
        // Temporary mock implementation
        Ok(Job {
            id: job.request_id, // Use request_id as job_id for now
            request_id: job.request_id,
            status: JobStatus::Queued,
            proof_bytes: job.proof_bytes,
            public_inputs: job.public_inputs,
            outputs_json: job.outputs_json,
            fee_bps: job.fee_bps,
            root_hash: job.root_hash,
            nullifier: job.nullifier,
            amount: job.amount,
            outputs_hash: job.outputs_hash,
            tx_id: None,
            solana_signature: None,
            error_message: None,
            retry_count: 0,
            max_retries: 3,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
        })
    }

    async fn get_job_by_id(&self, _id: Uuid) -> Result<Option<Job>, Error> {
        // Temporary mock - returns None
        Ok(None)
    }

    async fn get_job_by_request_id(&self, _request_id: Uuid) -> Result<Option<Job>, Error> {
        // Temporary mock - returns None  
        Ok(None)
    }

    async fn update_job_status(&self, _id: Uuid, _status: JobStatus) -> Result<(), Error> {
        // Temporary mock
        Ok(())
    }

    async fn update_job_processing(&self, _id: Uuid, _tx_id: Option<String>) -> Result<(), Error> {
        // Temporary mock
        Ok(())
    }

    async fn update_job_completed(&self, _id: Uuid, _tx_id: String, _signature: String) -> Result<(), Error> {
        // Temporary mock
        Ok(())
    }

    async fn update_job_failed(&self, _id: Uuid, _error: String) -> Result<(), Error> {
        // Temporary mock
        Ok(())
    }

    async fn increment_retry_count(&self, _id: Uuid) -> Result<(), Error> {
        // Temporary mock
        Ok(())
    }

    async fn get_queued_jobs(&self, _limit: i64) -> Result<Vec<Job>, Error> {
        // Temporary mock - returns empty vec
        Ok(vec![])
    }

    async fn get_jobs_by_status(&self, _status: JobStatus) -> Result<Vec<JobSummary>, Error> {
        // Temporary mock - returns empty vec
        Ok(vec![])
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
    async fn create_nullifier(&self, _nullifier: Vec<u8>, _job_id: Uuid) -> Result<(), Error> {
        // Temporary mock
        Ok(())
    }

    async fn exists_nullifier(&self, _nullifier: &[u8]) -> Result<bool, Error> {
        // Temporary mock - always returns false (no double spend detection for now)
        Ok(false)
    }

    async fn get_nullifier(&self, _nullifier: &[u8]) -> Result<Option<Nullifier>, Error> {
        // Temporary mock - returns None
        Ok(None)
    }

    async fn update_nullifier_block_info(&self, _nullifier: &[u8], _block_height: i64, _tx_signature: String) -> Result<(), Error> {
        // Temporary mock
        Ok(())
    }
} 