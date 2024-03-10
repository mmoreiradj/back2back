use std::{sync::Arc, time::Duration};

use back2back_lib::crd::B2BBackup;
use futures::StreamExt;
use k8s_openapi::api::batch::v1::CronJob;
use kube::{
    api::ListParams,
    runtime::{controller::Action, Controller},
    Api, Client, ResourceExt,
};
use thiserror::Error;
use tracing::{error, info};

/// Context of the controller
#[derive(Clone)]
pub struct Context {
    /// Kubernetes client
    client: Client,
}

#[derive(Error, Debug)]
pub enum ControllerError {
    #[error("Kube error: {0}")]
    KubeError(#[from] kube::Error),
}

pub type Result<T, E = ControllerError> = std::result::Result<T, E>;

async fn reconcile(b2b_backup: Arc<B2BBackup>, ctx: Arc<Context>) -> Result<Action> {
    info!("Reconciling b2b backup {:?}", b2b_backup.name_any());

    let client = ctx.client.clone();
    let b2b_backup: Arc<B2BBackup> = b2b_backup.clone();
    let b2b_backup_namespace = b2b_backup.namespace().unwrap_or("default".to_string());

    let cronjobs = Api::<CronJob>::namespaced(client, &b2b_backup_namespace);

    let lp = ListParams::default().labels(&format!(
        "b2b.moreiradj.fr/backup-name={}",
        &b2b_backup.name_any()
    ));
    let related_cronjobs = cronjobs.list_metadata(&lp).await?;

    if related_cronjobs.items.is_empty() {
        info!(
            "Cronjob not found for {:?}, creating it",
            b2b_backup.name_any()
        );

        let cronjob = b2b_backup.create_cronjob();

        cronjobs.create(&Default::default(), &cronjob).await?;
    }

    Ok(Action::requeue(Duration::from_secs(60)))
}

fn error_policy(b2b_backup: Arc<B2BBackup>, err: &ControllerError, _ctx: Arc<Context>) -> Action {
    error!("Error reconciling {:?}: {}", b2b_backup, err);

    Action::requeue(Duration::from_secs(60))
}

/// Initialize the controller (checks wether the CRD is present)
pub async fn run() -> Result<(), ControllerError> {
    info!("Initializing controller");
    let client = Client::try_default()
        .await
        .map_err(ControllerError::KubeError)?;

    let b2b_backups = Api::<B2BBackup>::all(client.clone());

    Controller::new(b2b_backups.clone(), Default::default())
        .run(reconcile, error_policy, Arc::new(Context { client }))
        .for_each(|_| futures::future::ready(()))
        .await;

    Ok(())
}
