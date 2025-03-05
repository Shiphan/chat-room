FROM php:8.1.31-apache

WORKDIR /var/www

# COPY conf/ conf/
COPY src/ html/

RUN a2enmod rewrite
