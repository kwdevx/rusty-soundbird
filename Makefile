-include .env
LOCAL_IMAGE="rusty-music-bot:latest"
	
.PHONY: dev
dev:
	@echo "[I] Start application in development env"
	docker compose up
	
# .PHONY: all
# all: login build retag push
#
# .PHONY: login
# login:
# 	@echo "[I] Start Login to ${IMAGE_REGISTRY} "
# 	aws ecr get-login-password --region ${CLOUD_REGION} | docker login --username AWS --password-stdin ${IMAGE_REGISTRY}
# 	@echo "[I] Login to ${IMAGE_REGISTRY} sucess"
#
.PHONY: build
build:
	@echo "[I] Start building application"
	docker build . -t ${LOCAL_IMAGE}
	@echo "[I] Build success"

# .PHONY: retag
# retag:
# 	@echo "[I]  Tagging built image"
# 	docker tag ${LOCAL_IMAGE} ${IMAGE_REGISTRY}/${LOCAL_IMAGE}    
#
# .PHONY : push
# push :
# 	@echo "Start pushing built image to container registry"
# 	docker push ${IMAGE_REGISTRY}/${LOCAL_IMAGE}
# 	@echo "Push success"
