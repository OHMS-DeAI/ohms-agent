use candid::{CandidType, Principal};
use ic_cdk::api::call::call;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct ChunkInfo {
    pub id: String,
    pub offset: u64,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum ModelState { Pending, Active, Deprecated }

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct ModelManifest {
    pub model_id: String,
    pub version: String,
    pub chunks: Vec<ChunkInfo>,
    pub digest: String,
    pub state: ModelState,
    pub uploaded_at: u64,
    pub activated_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct ModelMeta {
    pub family: String,
    pub arch: String,
    pub tokenizer_id: String,
    pub vocab_size: u32,
    pub ctx_window: u32,
    pub license: String,
}

pub struct ModelRepoClient;

impl ModelRepoClient {
    pub async fn get_manifest(canister_id: &str, model_id: &str) -> Result<ModelManifest, String> {
        let can_principal: Principal = canister_id.parse().map_err(|_| "invalid canister id")?;
        let arg = (model_id.to_string(),);
        let (opt_manifest,): (Option<ModelManifest>,) = call(can_principal, "get_manifest", arg)
            .await
            .map_err(|e| format!("xnet get_manifest failed: {:?}", e))?;
        opt_manifest.ok_or_else(|| "manifest not found".to_string())
    }

    pub async fn get_model_meta(canister_id: &str, model_id: &str) -> Result<ModelMeta, String> {
        let can_principal: Principal = canister_id.parse().map_err(|_| "invalid canister id")?;
        let arg = (model_id.to_string(),);
        let (opt_meta,): (Option<ModelMeta>,) = call(can_principal, "get_model_meta", arg)
            .await
            .map_err(|e| format!("xnet get_model_meta failed: {:?}", e))?;
        opt_meta.ok_or_else(|| "meta not found".to_string())
    }

    pub async fn get_chunk(canister_id: &str, model_id: &str, chunk_id: &str) -> Result<Vec<u8>, String> {
        let can_principal: Principal = canister_id.parse().map_err(|_| "invalid canister id")?;
        let arg = (model_id.to_string(), chunk_id.to_string());
        let (opt_bytes,): (Option<Vec<u8>>,) = call(can_principal, "get_chunk", arg)
            .await
            .map_err(|e| format!("xnet get_chunk failed: {:?}", e))?;
        opt_bytes.ok_or_else(|| "chunk not found".to_string())
    }
}

