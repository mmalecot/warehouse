# Instructions for building a Warehouse docker image.
# This image will use PostgreSQL as a database.
#
# You can build it using:
#   docker build -t warehouse-postgres .
#
# You can try it using:
#   docker-compose -f docker/warehouse-postgres/docker-compose.yml up -d
#
FROM alpine:3.12 as build

WORKDIR /usr/src/warehouse
COPY . .

RUN apk --no-cache add cargo rust postgresql-dev pacman-dev
RUN cargo build --release --features postgres --no-default-features

FROM alpine:3.12

COPY --from=build /usr/src/warehouse/target/release/warehouse /usr/bin/warehouse
COPY --from=build /usr/src/warehouse/resources /usr/share/warehouse

RUN apk --no-cache add bash libpq libgcc pacman

CMD ["/usr/bin/warehouse"]
