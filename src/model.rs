use std::ops;
use ::time::{format_description::well_known::Rfc3339, OffsetDateTime};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};


use anyhow::{bail, Context, Result};
use bytes::Bytes;
use futures_util::future::FutureExt;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    StatusCode,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use tokio::{
    sync::{oneshot, RwLock},
    time,
};
use tracing::{debug, error, info, warn};
use dav_server::fs::{DavDirEntry, DavMetaData, FsFuture, FsResult};
use serde_json::Value;



#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct LoginResponse {
    pub code: u32,
    pub message: String,
    pub submessage: String,
    pub data:LoginData,
}

#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct LoginRequest {
    pub token: String,
}

#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct LoginData {
    pub token: String,
    pub id: String,
}

#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct EncrypResponse {
    pub code: u32,
    pub message: String,
    pub submessage: String,
    pub data:EncryptData,
}

#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct EncryptData {
    pub Key: String,
}


#[derive(Debug, Clone)]
pub struct Credentials {
    pub token: String,
    pub key: String,
    pub uid: String,
}


#[derive(Debug, Clone, Serialize)]
pub struct CreateFolderRequest<'a> {
    pub kind: &'a str,
    pub name: &'a str,
    pub parent_id: &'a str,
}

#[derive(Debug, Clone, Serialize)]
pub struct DelFileRequest {
    pub ids: Vec<String>,
}


#[derive(Debug, Clone, Serialize)]
pub struct MoveFileRequest {
    pub ids: Vec<String>,
    pub to: MoveTo,
}


#[derive(Debug, Clone, Serialize)]
pub struct MoveTo {
    pub parent_id: String,
}

mod my_date_format {
    use chrono::{DateTime, Utc, TimeZone};
    use serde::{self, Deserialize, Serializer, Deserializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";
    pub fn serialize<S>(
        date: &DateTime<Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}


#[derive(Debug, Clone, Serialize)]
pub struct RenameFileRequest<'a>{
    pub name: &'a str,
}




#[derive(Debug, Clone, Deserialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub token_type: String,
}





#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    Folder,
    File,
}


#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct QuotaResponse {
    pub kind: String,
    pub expires_at: String,
    pub quota: Quota,
}

#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct Quota {
    pub kind: String,
    pub limit: u64,
    pub usage: u64,
    pub usage_in_trash: u64,
    pub play_times_limit: u64,
    pub play_times_usage:u64,
}


#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct ResultRequest {
    pub fileHash: String,
    pub accessToken: String,
}



#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct CallbackRequest {
    pub fileHashs: Vec<String>,
    pub accessToken: String,
}


#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct CallbackResponse {
    pub code: u64,
    pub message: String,
    pub submessage: String,
    pub data: Vec<CallBackData>,
    pub rsptime: u64,
}



#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct CallBackData {
    pub fileHash: String,
    pub fileCid: String,
    pub thumbnailCid: String,
    pub coverCid: String,
    pub cbState: u64,
}

#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct UploadInitRequest {
    pub hash: String,
    pub fileHash: String,
    pub fileName: String,
    pub fileSize: u64,
    pub fileCid: String,
    pub fileState:u64,
    pub parentId: String,
    pub chunkSize: u64,
    pub suffix:String,
    pub partList:Vec<Value>,
    pub accessToken: String,
}



#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct UploadInitResponse {
    pub code: u64,
    pub message: String,
    pub submessage: String,
    pub data: InitResponseData,
    pub rsptime: u64,
}


#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct InitResponseData {
    pub uploadEp: String,
    pub fileName: String,
    pub fileSize: u64,
    pub fileMimeType: String,
    pub fileExtension: String,
    pub chunkSize: u64,
    pub fileHash: String,
    pub fileCid: String,
    pub uploadState:u64,
}

#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct ObjProvider {
    pub provider: String,
}

#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct OssArgs {
    pub uploader:String,
    pub hash:String,
    pub chunkSize:u64,
}


#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct CompleteMultipartUpload {
    pub Part: Vec<PartInfo>,
}



#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct CompleteFileUpload {
    pub data: FileUploadInfo,
    pub status:u64,
}


#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct AddFileRequest {
    pub filePath: String,
    pub dirPath: Vec<String>,
    pub fileName: String,
    pub fileSize: u64,
    pub fileCid: String,
    pub fileType: u64,
    pub parentId: String,
    pub suffix: String,
    pub thumbnail: String,
    pub duration: u64,
    pub width: String,
    pub height: String,
}
#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct AddFileResponse {
    pub code: u64,
    pub message: String,
    pub submessage: String,
    pub count: u64,
    pub stime: u64,
}





#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct CompleteUploadRequest {
    pub appEnv: String,
    pub fileName: String,
    pub fileHash: String,
    pub accessToken: String,
    pub passThrough: String,
    pub noCallback: bool,
}


#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct SliceUploadResponse {
    pub code: u64,
    pub message: String,
    pub submessage: String,
    pub data: FileUploadInfo,
    pub rsptime: u64,
}


#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct FileUploadInfo {
    pub uploadEp: String,
    pub fileName: String,
    pub fileSize: u64,
    pub chunkSize: u64,
    pub uploadChunks: Vec<PartInfo>,
    pub fileCid: String,
    pub thumbnailCid: String,
    pub coverCid: String,
    pub uploadState: u64,
    pub fileMimeType: String,
    pub fileExtension: String,
    pub fileHash: String,
    pub queueExpireTs: u64,
}

#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct PartInfo {
    pub chunkIndex: u64,
    pub chunkSize: u64,
}

// #[derive(Debug, Serialize, Deserialize)]
// struct Example {
//     next: serde_json::Value,
// }



#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct UploadResponse {
    pub upload_type: String,
    pub resumable: Resumable,
    pub file: WebdavFile,
}



#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct PrepareFileResponse {
    pub data: PrepareInfo,
    pub status: u64,
}



#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct CompleteUploadResponse {
    pub data: String,
    pub status: u64,
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareInfo {
    pub next: u64,
    pub total: u64,
    pub wait: u64,
    pub uploading: u64,
    pub success: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SliceNextResult {
    Bool(bool),
    Int(i64),
}


#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct FileResponse {
    pub data: String,
    pub status: u64,
}

#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct FileUploadResponse {
    pub data: UploaderResponse,
    pub status: u64,
}

#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct UploaderResponse {
    pub utoken: String,
    pub uploader: String,
    pub src:String,
}








#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct Resumable {
    pub kind: String,
    pub provider: String,
    pub params: UploadParams,
}

#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct UploadParams {
    pub access_key_id: String,
    pub access_key_secret: String,
    pub bucket: String,
    pub endpoint: String,
    pub expiration: String,
    pub key: String,
    pub security_token: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct InitiateMultipartUploadResult {
    pub Bucket: String,
    pub Key: String,
    pub UploadId: String,
}




#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct WebdavFile {
    pub id: String,
    pub fileName: String,
    pub parentId: String,
    pub fileSize: String,
    pub isFolder: u64,
    pub fileCid:  Option<String>,
    #[serde(with = "my_date_format")]
    pub createTime: DateTime<Utc>,
    pub download_url: Option<String>,
    pub sha1: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesListRequest {
    pub fileType: Vec<String>,
    pub fileName: Value,
    pub pageNum: Value,
    pub pageSize: Value,
    pub parentId: Value,
    pub sortType: Value,
    pub sortFlag: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesListResponse {
    pub data: FilesList,
    pub code: Value,
    pub ts: Value,
    pub stime: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesList {
    pub data: Vec<WebdavFile>,
    pub pageNum: Value,
    pub pageSize: Value,
    pub count: Value,
    pub totalPage: Value,
}




impl DavMetaData for WebdavFile {
    fn len(&self) -> u64 {
        //self.size
        self.fileSize.parse::<u64>().unwrap()
    }

    fn modified(&self) -> FsResult<SystemTime> {
        let timestamp = self.createTime.timestamp();
        let system_time = UNIX_EPOCH + std::time::Duration::from_secs(timestamp as u64);
        Ok(system_time)
    }

    fn is_dir(&self) -> bool {
        //matches!(self.kind, String::from("drive#folder") )
        self.isFolder==1
    }

    fn created(&self) -> FsResult<SystemTime> {
        let timestamp = self.createTime.timestamp();
        let system_time = UNIX_EPOCH + std::time::Duration::from_secs(timestamp as u64);
        Ok(system_time)
    }
}

impl DavDirEntry for WebdavFile {
    fn name(&self) -> Vec<u8> {
        self.fileName.as_bytes().to_vec()
    }

    fn metadata(&self) -> FsFuture<Box<dyn DavMetaData>> {
        async move { Ok(Box::new(self.clone()) as Box<dyn DavMetaData>) }.boxed()
    }
}

impl WebdavFile {
    pub fn new_root() -> Self {
        Self {
            id: "0".to_string(),
            isFolder:1,
            fileName: "".to_string(),
            parentId: "".to_string(),
            fileSize: "0".to_string(),
            createTime: chrono::offset::Utc::now(),
            fileCid: None,
            sha1: None,
            download_url:None,
        }
    }
}


