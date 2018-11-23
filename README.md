# Image Resizer In Rust

A file hosting server that adjusts images in real time.

**Warning: Do not use in a production environment, this project needs a lot of improvement.**

Feature preview: [irir.bluerain.io](https://irir.bluerain.io)

## Usage

### Running

````bash
cargo run -- -o ./originals -O ./outputs
````

### Basic usage (resize)

1. I need to adjust the `height` of the image to `600`  
  [http://localhost:8080/display/h600/ferris.png](http://localhost:8080/display/h600/ferris.png)
1. Then I set the `width` to `400`  
  [http://localhost:8080/display/h600w400/ferris.png](http://localhost:8080/display/h600w400/ferris.png)
1. I want to use the query parameters  
  [http://localhost:8080/display/ferris.png?w=400&h=600](http://localhost:8080/display/ferris.png?w=400&h=600)

### Notes

* Missing one size parameter will preserve the aspect ratio
* No size parameter will preserve the original size (But usually compresses the file size)

### More features

* blur
* crop
* ……
* ~~to be developed~~
