# Usage default features:
# tilt up
#
# Usage with features:
# tilt up telemetry
# config.define_string("features", args=True)
# cfg = config.parse()
# features = cfg.get('features', "")
print("compiling...")

docker_build('mmoreiradj/back2back-controller', '.', dockerfile='Dockerfile')
k8s_yaml('yaml/crd.yaml')
k8s_yaml('yaml/deployment.yaml')
k8s_resource('back2back-controller', port_forwards="8080")
