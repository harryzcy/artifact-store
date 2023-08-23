# Production Deployment

## Docker

```shell
# change the permission for non-root owner (65532)
chown -R 65532:65532 /PATH/TO/DATA

docker run -v /PATH/TO/DATA:/data -p 3001:3001 ghcr.io/harryzcy/artifact-store
```

## Kubernetes Manifest

It can be deployed via StatefulSet.

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: artifact-store
spec:
  selector:
    matchLabels:
      app: artifact-store
  serviceName: artifact-store
  replicas: 1
  template:
    metadata:
      labels:
        app: artifact-store
    spec:
      initContainers:
        - name: volume-mount-user
          image: alpine
          command: ["/bin/sh"]
          args:
            - -c
            - >-
              chown -R 65532:65532 /data
          volumeMounts:
            - name: artifact-store-data
              mountPath: /data
      containers:
        - name: artifact-store
          image: ghcr.io/harryzcy/artifact-store:latest
          ports:
            - containerPort: 3001
              name: http
          volumeMounts:
            - name: artifact-store-data
              mountPath: /data
          resources:
            requests:
              cpu: 100m
              memory: 256Mi
            limits:
              cpu: 100m
              memory: 256Mi
  volumeClaimTemplates:
    - metadata:
        name: artifact-store-data
      spec:
        accessModes:
          - ReadWriteOnce
        resources:
          requests:
            storage: 1Gi
```
