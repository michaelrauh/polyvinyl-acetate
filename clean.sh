helm uninstall postgres-k
helm uninstall rabbit-k
kubectl delete deploy pvac-web
kubectl delete deploy pvac-relay
kubectl delete deploy pvac-worker
kubectl delete pod --all
kubectl delete pvc --all
kubectl delete pv --all
kubectl delete services web-entrypoint
