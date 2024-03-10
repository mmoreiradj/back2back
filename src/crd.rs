use std::collections::BTreeMap;

use k8s_openapi::{
    api::{
        batch::v1::{CronJob, CronJobSpec, JobSpec, JobTemplateSpec},
        core::v1::{
            Capabilities, Container, EnvVar, PodSpec, PodTemplateSpec, SeccompProfile,
            SecurityContext,
        },
    },
    apimachinery::pkg::apis::meta::v1::OwnerReference,
};
use kube::{api::ObjectMeta, CustomResource, ResourceExt};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
pub struct PostgresAuth {
    pub password: String,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
pub struct PostgresConfig {
    pub host: String,
    pub port: u16,
    pub auth: PostgresAuth,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
pub struct S3Auth {
    pub access: String,
    pub secret: String,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
pub struct S3Config {
    pub endpoint: Option<String>,
    pub bucket: String,
    pub auth: S3Auth,
    pub region: Option<String>,
}

#[derive(CustomResource, Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[kube(
    group = "b2b.moreiradj.fr",
    version = "v1",
    kind = "B2BBackup",
    namespaced
)]
pub struct B2BBackupSpec {
    pub postgres: PostgresConfig,
    pub s3: S3Config,
    pub schedule: String,
}

const BACKUP_IMAGE_NAME: &str = "ghcr.io/mmoreiradj/back2back-postgres-backup:latest";

impl B2BBackup {
    pub fn build_owner_reference(&self) -> OwnerReference {
        OwnerReference {
            api_version: "b2b.moreiradj.fr/v1".to_string(),
            kind: "B2BBackup".to_string(),
            name: self.name_any(),
            // TODO: Handle UID not found
            uid: self.uid().expect("No UID found"),
            controller: Some(true),
            block_owner_deletion: Some(true),
        }
    }

    pub fn default_labels(&self) -> BTreeMap<String, String> {
        let mut labels = BTreeMap::new();
        labels.insert("app.kubernetes.io/managed-by".to_string(), self.name_any());
        labels.insert("b2b.moreiradj.fr/backup-name".to_string(), self.name_any());
        labels
    }

    pub fn create_cronjob(&self) -> CronJob {
        let env = vec![
            EnvVar {
                name: "PGUSER".to_string(),
                value: Some("postgres".to_string()),
                value_from: None,
            },
            EnvVar {
                name: "PGPASSWORD".to_string(),
                value: Some(self.spec.postgres.auth.password.to_string()),
                value_from: None,
            },
            EnvVar {
                name: "PGHOST".to_string(),
                value: Some(self.spec.postgres.host.to_string()),
                value_from: None,
            },
            EnvVar {
                name: "PGPORT".to_string(),
                value: Some(self.spec.postgres.port.to_string()),
                value_from: None,
            },
            EnvVar {
                name: "PGDUMP_DIR".to_string(),
                value: Some("/tmp".to_string()),
                value_from: None,
            },
            EnvVar {
                name: "S3_ENDPOINT".to_string(),
                value: self.spec.s3.endpoint.clone(),
                value_from: None,
            },
            EnvVar {
                name: "AWS_ACCESS_KEY_ID".to_string(),
                value: Some(self.spec.s3.auth.access.to_string()),
                value_from: None,
            },
            EnvVar {
                name: "AWS_SECRET_ACCESS_KEY".to_string(),
                value: Some(self.spec.s3.auth.secret.to_string()),
                value_from: None,
            },
            EnvVar {
                name: "AWS_DEFAULT_REGION".to_string(),
                value: self.spec.s3.region.clone(),
                value_from: None,
            },
        ];

        CronJob {
            metadata: ObjectMeta {
                name: Some(self.name_any()),
                namespace: Some(self.namespace().unwrap_or("default".to_string())),
                owner_references: Some(vec![self.build_owner_reference()]),
                labels: Some(self.default_labels()),
                ..Default::default()
            },
            spec: Some(CronJobSpec {
                schedule: self.spec.schedule.to_string(),
                concurrency_policy: Some("Replace".to_string()),
                job_template: JobTemplateSpec {
                    metadata: Some(ObjectMeta {
                        labels: Some(self.default_labels()),
                        name: Some(format!("{}-{}", self.name_any(), "pgdump")),
                        ..Default::default()
                    }),
                    spec: Some(JobSpec {
                        template: PodTemplateSpec {
                            spec: Some(PodSpec {
                                containers: vec![Container {
                                    env: Some(env),
                                    image: Some(BACKUP_IMAGE_NAME.to_string()),
                                    image_pull_policy: Some("IfNotPresent".to_string()),
                                    name: format!("{}-pgdump", self.name_any()),
                                    security_context: Some(SecurityContext {
                                        allow_privilege_escalation: Some(false),
                                        capabilities: Some(Capabilities {
                                            drop: Some(vec!["ALL".to_string()]),
                                            ..Default::default()
                                        }),
                                        read_only_root_filesystem: Some(false),
                                        run_as_group: Some(0),
                                        run_as_non_root: Some(false),
                                        run_as_user: Some(0),
                                        seccomp_profile: Some(SeccompProfile {
                                            type_: "RuntimeDefault".to_string(),
                                            ..Default::default()
                                        }),
                                        ..Default::default()
                                    }),
                                    termination_message_path: Some(
                                        "/dev/termination-log".to_string(),
                                    ),
                                    termination_message_policy: Some("File".to_string()),
                                    ..Default::default()
                                }],
                                restart_policy: Some("OnFailure".to_string()),
                                ..Default::default()
                            }),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                },
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

pub fn generate_random_string(length: usize) -> String {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect();
    rand_string
}
