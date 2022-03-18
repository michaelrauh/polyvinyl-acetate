kubectl config use-context docker-desktop

source ./build_common.sh
kubectl apply -f web.yaml
kubectl apply -f relay.yaml