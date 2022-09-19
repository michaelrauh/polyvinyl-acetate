helm uninstall postgres-k
helm uninstall rabbit-k

kubectl delete -f web-prod.yaml
kubectl delete -f relay-prod.yaml
kubectl delete -f worker-prod.yaml
kubectl delete -f simplest.yaml