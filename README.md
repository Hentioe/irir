# Image Resizer In Rust

A file server that can resize images in real time.

## Usage

````bash
# Defining parameters
port=8080
origin_dir=/home/static/images
output_dir=/home/static/caches

# Running
irirserver --dir-origin ${origin_dir} --dir-output ${output_dir} \
-p 8080
````

Note: **Currently in the Alpha stage, but basic features are available**