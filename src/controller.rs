use std::{sync::Arc, time::Duration};

use back2back_lib::crd::{B2BBackup, B2BBackupStatus};
use futures::StreamExt;
use k8s_openapi::api::{batch::v1::CronJob, core::v1::Pod};
use kube::{
    api::{Patch, PatchParams},
    runtime::{controller::Action, Controller},
    Api, Client, ResourceExt,
};
use serde_json::json;
use thiserror::Error;
use tracing::{debug, error, info};

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
    let b2b_backups = Api::<B2BBackup>::namespaced(client.clone(), &b2b_backup_namespace);

    let cronjobs = Api::<CronJob>::namespaced(client.clone(), &b2b_backup_namespace);

    let cronjob = match cronjobs.get(&b2b_backup.name_any()).await {
        Ok(cronjob) => cronjob,
        Err(_) => {
            debug!(
                "Cronjob not found for {:?}, creating it",
                b2b_backup.name_any()
            );

            let cronjob = b2b_backup.create_cronjob();

            cronjobs.create(&Default::default(), &cronjob).await?;
            return Ok(Action::requeue(Duration::from_secs(60)));
        }
    };

    let cronjob_status = match cronjob.status {
        Some(status) => status,
        None => {
            debug!("Cronjob status not found for {:?}", b2b_backup.name_any());
            return Ok(Action::requeue(Duration::from_secs(60)));
        }
    };

    let cronjob_last_successful_time = match cronjob_status.last_successful_time {
        Some(last_schedule) => last_schedule,
        None => {
            debug!(
                "Cronjob last schedule not found for {:?}",
                b2b_backup.name_any()
            );
            return Ok(Action::requeue(Duration::from_secs(60)));
        }
    };

    let status = match &b2b_backup.status {
        Some(status) => status,
        None => {
            let new_status = Patch::Apply(json!({
                "apiVersion": "b2b.moreiradj.fr/v1",
                "kind": "B2BBackup",
                "status": B2BBackupStatus {
                    cronjob_last_successful_time: Some(cronjob_last_successful_time),
                    test_pod_last_schedule_time: None,
                    test_pod_last_successful_time: None,
                }

            }));

            let ps = PatchParams::apply(&b2b_backup_namespace).force();
            let _ = b2b_backups
                .patch_status(&b2b_backup.name_any(), &ps, &new_status)
                .await?;

            return Ok(Action::requeue(Duration::from_secs(60)));
        }
    };

    // Schedule a pod to test the backup
    if status.cronjob_last_successful_time != status.test_pod_last_schedule_time {
        debug!(
            "Scheduling a pod to test the backup for {:?}",
            b2b_backup.name_any()
        );

        let pods = Api::<Pod>::namespaced(client.clone(), &b2b_backup_namespace);
        let pod = b2b_backup.create_test_postgres_pod();

        pods.create(&Default::default(), &pod).await?;

        let cronjob_last_successful_time = cronjob_last_successful_time.clone();

        let new_status = Patch::Apply(json!({
            "apiVersion": "b2b.moreiradj.fr/v1",
            "kind": "B2BBackup",
            "status": B2BBackupStatus {
                cronjob_last_successful_time: Some(cronjob_last_successful_time.clone()),
                test_pod_last_schedule_time: Some(cronjob_last_successful_time),
                test_pod_last_successful_time: None,
            }
        }));

        let ps = PatchParams::apply(&b2b_backup_namespace).force();
        let _ = b2b_backups
            .patch_status(&b2b_backup.name_any(), &ps, &new_status)
            .await?;
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
