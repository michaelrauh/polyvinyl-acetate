helm uninstall postgres-k
kubectl delete deploy pvac-web
kubectl delete pod --all
kubectl delete pvc --all
kubectl delete pv --all
kubectl delete services web-entrypoint
