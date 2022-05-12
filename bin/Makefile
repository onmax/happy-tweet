build:
	docker build -t happy-tweet-image .

run:
	docker container stop happy-tweet
	docker container rm happy-tweet 
	docker run --name happy-tweet happy-tweet-image
	docker cp happy-tweet:/output.json .