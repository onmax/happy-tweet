build:
	docker build -t happy-tweet-image .

run:
	docker stop happy-tweet || true && docker rm happy-tweet || true
	docker run --name happy-tweet happy-tweet-image
	docker cp happy-tweet:/output.json .