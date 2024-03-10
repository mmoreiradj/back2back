# Usage default features:
# tilt up
#
# Usage with features:
# tilt up telemetry
# config.define_string("features", args=True)
# cfg = config.parse()
# features = cfg.get('features', "")
print("compiling...")

local_resource('compile', 'just compile', deps=['./src', 'Cargo.toml', 'Cargo.lock'])
docker_build('mmoreiradj/back2back-controller', '.', dockerfile='Dockerfile')
k8s_yaml('yaml/crd.yaml')
k8s_yaml('yaml/deployment.yaml')
k8s_yaml('yaml/minio-deployment.yaml')
k8s_resource('back2back-controller', port_forwards="8080")
k8s_resource('back2back-minio-test', port_forwards=["9000", "9001"])
