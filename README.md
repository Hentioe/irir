# Image Resizer In Rust

A file server that can resize images in real time.

## Usage

### Running

````bash
cargo run -- -o ./originals -O ./outputs
````

### Access

1. I need to scale the `height` of the image to `250`  
  [http://localhost:8080/display/h250/jojo_01.jpg](http://localhost:8080/display/h250/jojo_01.jpg)
1. Then I set the `width` to `300`  
  [http://localhost:8080/display/h250w300/jojo_01.jpg](http://localhost:8080/display/h250w300/jojo_01.jpg)
1. I want to use the query parameters  
  [http://localhost:8080/display/jojo_01.jpg?w=300&h=250](http://localhost:8080/display/jojo_01.jpg?w=300&h=250)

## Notes

* Missing one size parameter will preserve the aspect ratio
* No size parameter will retain the original size (But usually compresses the file size)