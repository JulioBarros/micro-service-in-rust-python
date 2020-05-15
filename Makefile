

run-through: get-quotes
	curl http://localhost:8000/
	curl "http://localhost:8000/instruments/aapl?small=7&large=14&duration=60"
	curl "http://localhost:8000/instruments/aapl/stats?small=7&large=14&duration=60"

get-instruments:
	curl "http://localhost:8000/"

get-quotes:
	curl "http://localhost:8000/instruments/aapl?duration=10"

get-stats:
	curl "http://localhost:8000/instruments/aapl/stats?duration=5"

get-crossovers:
	curl "http://localhost:8000/instruments/aapl/crossovers?small=7&large=14&duration=60"	

postgres:
	docker run -it -p 5432:5432 --rm -e POSTGRES_PASSWORD=postgres -d postgres

kill-pg:
	docker kill `docker ps| grep postgres | awk '{print $1}'`

psql:
	docker run -it --rm  postgres psql -h host.docker.internal -U postgres postgres
