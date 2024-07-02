DOCKER_TAG := trusted-core:latest

.PHONY: docker_run docker_build

# docker build  -t ${DOCKER_TAG} --target build .
docker_build:
	docker build -t ${DOCKER_TAG} --target build .

docker_run:
	docker run --rm -it -v ${PWD}:/mnt --name trusted-core ${DOCKER_TAG} bash
