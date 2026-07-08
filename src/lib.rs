//! Typed request and reply contract between `mind` and `mind-judge`.
//!
//! The binary wire is rkyv-backed through `signal-frame`. NOTA projection is an
//! edge feature for clients, tests, and tools; it is not the component-to-
//! component transport.

#![forbid(unsafe_code)]

use thiserror::Error;

pub const SIGNAL_SCHEMA_SOURCE: &str = include_str!("../schema/signal.schema");

pub type MindJudgeFrame = signal_frame::ExchangeFrame<MindJudgeRequest, MindJudgeReply>;
pub type MindJudgeFrameBody = signal_frame::ExchangeFrameBody<MindJudgeRequest, MindJudgeReply>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("mind judge contract value is empty")]
    EmptyValue,
}

#[cfg_attr(
    feature = "nota-text",
    derive(nota::NotaDecode, nota::NotaDecodeTraced, nota::NotaEncode)
)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum MindJudgeRequest {
    JudgeKnowledge(KnowledgeJudgePacket),
}

impl signal_frame::RequestPayload for MindJudgeRequest {}

impl signal_frame::LogVariant for MindJudgeRequest {
    fn log_variant(&self) -> u64 {
        match self {
            Self::JudgeKnowledge(_) => 1,
        }
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(nota::NotaDecode, nota::NotaDecodeTraced, nota::NotaEncode)
)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum MindJudgeReply {
    KnowledgeJudged(KnowledgeJudgeResponse),
    RequestRejected(MindJudgeRequestRejection),
}

#[cfg_attr(
    feature = "nota-text",
    derive(nota::NotaDecode, nota::NotaDecodeTraced, nota::NotaEncode)
)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct KnowledgeJudgePacket {
    pub domain: signal_domain::Domain,
    pub statement: TextBody,
    pub relevant_neighbors: Vec<KnowledgeRecord>,
}

impl KnowledgeJudgePacket {
    pub fn new(
        domain: signal_domain::Domain,
        statement: TextBody,
        relevant_neighbors: Vec<KnowledgeRecord>,
    ) -> Self {
        Self {
            domain,
            statement,
            relevant_neighbors,
        }
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(nota::NotaDecode, nota::NotaDecodeTraced, nota::NotaEncode)
)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct KnowledgeRecord {
    pub identity: KnowledgeIdentity,
    pub domain: signal_domain::Domain,
    pub statement: TextBody,
}

impl KnowledgeRecord {
    pub fn new(
        identity: KnowledgeIdentity,
        domain: signal_domain::Domain,
        statement: TextBody,
    ) -> Self {
        Self {
            identity,
            domain,
            statement,
        }
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(nota::NotaDecode, nota::NotaDecodeTraced, nota::NotaEncode)
)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct KnowledgeJudgeResponse {
    pub verdict: KnowledgeJudgeVerdict,
    pub diagnostic_message: Option<TextBody>,
}

impl KnowledgeJudgeResponse {
    pub fn new(verdict: KnowledgeJudgeVerdict, diagnostic_message: Option<TextBody>) -> Self {
        Self {
            verdict,
            diagnostic_message,
        }
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(nota::NotaDecode, nota::NotaDecodeTraced, nota::NotaEncode)
)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum KnowledgeJudgeVerdict {
    Accept,
    Reject(KnowledgeRejectionReason),
}

#[cfg_attr(
    feature = "nota-text",
    derive(nota::NotaDecode, nota::NotaDecodeTraced, nota::NotaEncode)
)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum KnowledgeRejectionReason {
    NotKnowledge,
    PrivateOrUnauthorized,
    MeaningUnclear,
    SemanticDuplicate(KnowledgeIdentity),
    ConflictsAcceptedKnowledge(Vec<KnowledgeIdentity>),
    WrongDomain(signal_domain::Domain),
    NeedsMoreSpecificShape,
}

#[cfg_attr(
    feature = "nota-text",
    derive(nota::NotaDecode, nota::NotaDecodeTraced, nota::NotaEncode)
)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MindJudgeRequestRejection {
    pub reason: MindJudgeRequestRejectionReason,
    pub message: TextBody,
}

impl MindJudgeRequestRejection {
    pub fn new(reason: MindJudgeRequestRejectionReason, message: TextBody) -> Self {
        Self { reason, message }
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(nota::NotaDecode, nota::NotaDecodeTraced, nota::NotaEncode)
)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum MindJudgeRequestRejectionReason {
    InvalidRequest,
    ConfigurationUnavailable,
    ProviderUnavailable,
    ProviderRejected,
    ResponseFormatFailure,
}

macro_rules! non_empty_text_type {
    ($name:ident) => {
        #[cfg_attr(
            feature = "nota-text",
            derive(nota::NotaDecode, nota::NotaDecodeTraced, nota::NotaEncode)
        )]
        #[derive(
            rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq,
        )]
        pub struct $name(pub String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, Error> {
                let value = value.into();
                if value.is_empty() {
                    return Err(Error::EmptyValue);
                }
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }
    };
}

non_empty_text_type!(KnowledgeIdentity);
non_empty_text_type!(TextBody);
