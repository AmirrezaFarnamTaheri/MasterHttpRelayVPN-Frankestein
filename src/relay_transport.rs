use std::sync::Arc;

use crate::config::{Config, Mode};
use crate::domain_fronter::DomainFronter;
use crate::vercel_edge::{VercelEdgeRelay, VercelRelayError};

#[derive(Clone)]
pub enum RelayTransport {
    AppsScript(Arc<DomainFronter>),
    VercelEdge(Arc<VercelEdgeRelay>),
}

impl RelayTransport {
    pub fn new(
        config: &Config,
        apps_script: Option<Arc<DomainFronter>>,
    ) -> Result<Option<Arc<Self>>, VercelRelayError> {
        match config.mode_kind() {
            Ok(Mode::AppsScript | Mode::Full) => {
                Ok(apps_script.map(|f| Arc::new(Self::AppsScript(f))))
            }
            Ok(Mode::VercelEdge) => Ok(Some(Arc::new(Self::VercelEdge(Arc::new(
                VercelEdgeRelay::new(config)?,
            ))))),
            Ok(Mode::Direct) | Err(_) => Ok(None),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::AppsScript(_) => "Apps Script",
            Self::VercelEdge(_) => "Serverless JSON",
        }
    }

    pub async fn relay(
        &self,
        method: &str,
        url: &str,
        headers: &[(String, String)],
        body: &[u8],
    ) -> Vec<u8> {
        match self {
            Self::AppsScript(f) => f.relay(method, url, headers, body).await,
            Self::VercelEdge(v) => v.relay(method, url, headers, body).await,
        }
    }

    pub async fn relay_parallel_range(
        &self,
        method: &str,
        url: &str,
        headers: &[(String, String)],
        body: &[u8],
    ) -> Vec<u8> {
        match self {
            Self::AppsScript(f) => f.relay_parallel_range(method, url, headers, body).await,
            Self::VercelEdge(v) => {
                tracing::debug!(
                    "vercel_edge: range-parallel path not enabled; using single JSON relay"
                );
                v.relay(method, url, headers, body).await
            }
        }
    }
}
