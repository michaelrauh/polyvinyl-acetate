kubectl port-forward $(kubectl get pods -l=app="jaeger" -o name) 16686:16686