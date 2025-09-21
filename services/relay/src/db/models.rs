use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::JsonValue;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "job_status", rename_all = "lowercase")]
pub enum JobStatus {
    Queued,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Queued => write!(f, "queued"),
            JobStatus::Processing => write!(f, "processing"),
            JobStatus::Completed => write!(f, "completed"),
            JobStatus::Failed => write!(f, "failed"),
            JobStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Job {
    pub id: Uuid,
    pub request_id: Uuid,
    pub status: JobStatus,
    
    // Request data
    pub proof_bytes: Vec<u8>,
    pub public_inputs: Vec<u8>,
    pub outputs_json: JsonValue,
    pub fee_bps: i16,
    
    // Extracted public inputs for indexing
    pub root_hash: Vec<u8>,
    pub nullifier: Vec<u8>,
    pub amount: i64,
    pub outputs_hash: Vec<u8>,
    
    // Processing results
    pub tx_id: Option<String>,
    pub solana_signature: Option<String>,
    pub error_message: Option<String>,
    pub retry_count: i32,
    pub max_retries: i32,
    
    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct CreateJob {
    pub request_id: Uuid,
    pub proof_bytes: Vec<u8>,
    pub public_inputs: Vec<u8>,
    pub outputs_json: JsonValue,
    pub fee_bps: i16,
    pub root_hash: Vec<u8>,
    pub nullifier: Vec<u8>,
    pub amount: i64,
    pub outputs_hash: Vec<u8>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Nullifier {
    pub nullifier: Vec<u8>,
    pub job_id: Uuid,
    pub block_height: Option<i64>,
    pub tx_signature: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JobSummary {
    pub request_id: Uuid,
    pub status: JobStatus,
    pub tx_id: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<Job> for JobSummary {
    fn from(job: Job) -> Self {
        Self {
            request_id: job.request_id,
            status: job.status,
            tx_id: job.tx_id,
            error_message: job.error_message,
            created_at: job.created_at,
            completed_at: job.completed_at,
        }
    }
} 