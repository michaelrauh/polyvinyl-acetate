doctl k c d pvac-cluster
kubectl config use-context docker-desktop
sleep 5

VOLUME_ID=$(doctl compute volume list -o json | jq -r '.[0].id')
doctl compute volume delete $VOLUME_ID
