_default:
  just --list

docker-build:
  docker build \
    --tag asciicat \
    --target app \
    --ssh default \
    --secret id=shipyard-token,src=secrets/shipyard \
    .

deploy: docker-build
  fly deploy --local-only
