doctl k c d pvac-cluster

VOLUME_ID=$(doctl compute volume list -o json | jq -r '.[0].id')
doctl compute volume delete $VOLUME_ID

VOLUME_ID=$(doctl compute volume list -o json | jq -r '.[0].id')
doctl compute volume delete $VOLUME_ID

kubectl config use-context docker-desktop