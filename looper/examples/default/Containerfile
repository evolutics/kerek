FROM k8s.gcr.io/echoserver:1.10

LABEL info.evolutics.wheelsticks.container-names='web'
LABEL info.evolutics.wheelsticks.health-check='curl --fail localhost:8080 || exit 1'
LABEL info.evolutics.wheelsticks.networks='main'
LABEL info.evolutics.wheelsticks.port-mappings='80:8080'
