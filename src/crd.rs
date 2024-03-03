use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
pub struct PostgresAuth {
    pub username: String,
    pub password: String,
    pub database: String,
}

#[derive(CustomResource, Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[kube(
    group = "b2b.moreiradj.fr",
    version = "v1",
    kind = "B2BBackup",
    namespaced
)]
pub struct B2BPostgresBackupSpec {
    host: String,
    port: u16,
    auth: PostgresAuth,
    schedule: String,
}
