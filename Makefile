include .env
export

#### ---- CHECKING FILES DEPENDENCIES ---- ####
ifndef SURREAL_DB_USER
ifndef SURREAL_DB_PASSWORD
$(shell echo "DB_USER=\"login\"\nDB_PASSWORD=\"token\"" > .env)
include .env
endif
endif


#### --- HELP --- ####
# This will output the help for each task
# thanks to https://marmelab.com/blog/2016/02/29/auto-documented-makefile.html
.PHONY: help
help: ### Display this help screen
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

.PHONY: docker_compose_start
docker_compose_start:  ### - run docker compose up command with detached option
	docker compose -f ./deployments/docker-compose.yaml up -d

.PHONY: docker_compose_down
docker_compose_down:  ### - stop docker compose running containers setup
	docker compose -f ./deployments/docker-compose.yaml down

.PHONY: docker_compose_teardown
docker_compose_teardown:  ### - stop and delete docker compose running containers setup
	docker compose -f ./deployments/docker-compose.yaml down -v

.PHONY: run
run:  ### - run instance of flowlocker
	cargo run -p flowlocker


### cargo test -v --package lib-query_builder test_new -- --nocapture
