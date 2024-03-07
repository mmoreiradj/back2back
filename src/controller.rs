use std::{sync::Arc, time::Duration};

use back2back_lib::crd::B2BBackup;
use futures::StreamExt;
use kube::{
    runtime::{controller::Action, Controller},
    Api, Client,
};
use thiserror::Error;
use tracing::{error, info};
/// Context of the controller
#[derive(Clone)]
pub struct Context {}

#[derive(Error, Debug)]
pub enum ControllerError {
    #[error("Kube error: {0}")]
    KubeError(#[from] kube::Error),
}

pub type Result<T, E = ControllerError> = std::result::Result<T, E>;

async fn reconcile(b2b_backup: Arc<B2BBackup>, _ctx: Arc<Context>) -> Result<Action> {
    info!("Reconciling {:?}", b2b_backup);

    Ok(Action::requeue(Duration::from_secs(60)))
}

fn error_policy(b2b_backup: Arc<B2BBackup>, err: &ControllerError, _ctx: Arc<Context>) -> Action {
    error!("Error reconciling {:?}: {}", b2b_backup, err);

    Action::requeue(Duration::from_secs(60))
}

/// Initialize the controller (checks wether the CRD is present)
pub async fn run() -> Result<(), ControllerError> {
    let client = Client::try_default()
        .await
        .map_err(ControllerError::KubeError)?;

    let b2b_backups = Api::<B2BBackup>::all(client.clone());

    Controller::new(b2b_backups.clone(), Default::default())
        .run(reconcile, error_policy, Arc::new(Context {}))
        .for_each(|_| futures::future::ready(()))
        .await;

    Ok(())
}
