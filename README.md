# Image Resizer In Rust

A file server that can resize images in real time.

## Usage

### Running

````bash
cargo run -- -o ./originals -O ./outputs
````

### Access

1. I need to compress a image with a `height` of `100`  
  [http://localhost:8080/display/h100/jojo_01.jpg](http://localhost:8080/display/h100/jojo_01.jpg)
1. I need to compress a image with a `width` of `250`  
  [http://localhost:8080/display/w250/jojo_01.jpg](http://localhost:8080/display/w250/jojo_01.jpg)
3. I want to use the query parameters  
  [http://localhost:8080/display/jojo_01.jpg?w=150](http://localhost:8080/display/jojo_01.jpg?w=150)

Note: **Currently in the Alpha stage, but basic features are available**