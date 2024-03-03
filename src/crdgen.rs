use back2back_lib::crd::B2BBackup;
use kube::CustomResourceExt;

fn main() {
    print!("{}", serde_yaml::to_string(&B2BBackup::crd()).unwrap());
}
