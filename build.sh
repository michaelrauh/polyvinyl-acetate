source ./build_common.sh
kubectl apply -f web.yaml
kubectl apply -f relay.yaml
kubectl apply -f worker.yaml