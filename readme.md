# Bicycle GPS

## Get OSM data

Get an export for the area you are interested in.
http://download.geofabrik.de/

Install Postgres and [Osmosis](http://wiki.openstreetmap.org/wiki/Osmosis).
Import the data with pgsnapshot.

    $ createdb osmosis
    $ psql osmosis
    > create extension hstore;
    > create extension postgis;
    $ psql osmosis < /usr/share/osmosis/script/pgsnapshot_schema_0.6.sql
    $ psql osmosis < /usr/share/osmosis/script/pgsnapshot_schema_0.6_bbox.sql
    $ osmosis --read-pbf ~/netherlands-latest.osm.pbf --tf accept-ways highway=* --used-node --bounding-box --write-pgsql database=osmosis
