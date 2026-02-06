docker-up:
	@echo "🐘 Starting PostgreSQL containers..."
	@mkdir -p postgresql/pgdata
	@cd postgresql && docker compose --env-file .env up -d

# PostgreSQL 준비 대기
docker-wait:
	@echo "⏳ Waiting for PostgreSQL to be ready..."
	@until docker compose -f postgresql/docker-compose.yml --env-file postgresql/.env exec -T db pg_isready -U postgres > /dev/null 2>&1; do \
		sleep 1; \
	done
	@echo "✅ PostgreSQL is ready!"

# PostgreSQL 컨테이너 중지 및 삭제
docker-down:
	@echo "🛑 Stopping PostgreSQL containers..."
	@cd postgresql && docker compose --env-file .env down

# 완전 정리 (볼륨 포함)
clean-docker:
	@echo "🧹 Cleaning up PostgreSQL containers and volumes..."
	@cd postgresql && docker compose --env-file .env down -v
	@echo "🗑️  Removing pgdata directory..."
	@rm -rf postgresql/pgdata
