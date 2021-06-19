#!/usr/bin/env sh
#wget -O swaggerui.tar.gz https://github.com/swagger-api/swagger-ui/archive/refs/tags/v3.50.0.tar.gz
mkdir -p pages
cd pages
#tar -xvf ../swaggerui.tar.gz --strip-components=2 swagger-ui-3.50.0/dist
cp -r ../swagger-ui/* ./
cp ../swagger.json ./
cp ../*.schema.json ./
cp -r ../examples ./
cp -r ../schemas ./
