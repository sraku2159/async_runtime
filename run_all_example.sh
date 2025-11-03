#! /bin/bash

for i in $(ls examples | sed 's/.rs//g'); do cargo run --example $i; done
