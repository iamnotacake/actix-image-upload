### My first attempt on implementing REST API with actix-web


Upload single/multiple image(s):
```
-> POST /upload
   encoding: multipart/form-data
   data: image=(file content)
   data: image=(file2 content)
   data: image=(file3 content)

<- 200 OK
   or 415 Unsupported Media Type if uploaded file MIME type was not an image
   or 400 Bad Request if "name" was not "image" in Content-Disposition header
   content-type: application/json
   body: ["WgyLB9IKorGE","RoydCP3nVXkI"] # list of images' IDs that were successfully uploaded
```
Example usage:
* `./example-usage/upload-single-with-curl.sh`
* `./example-usage/upload-multi-with-curl.sh`
* `./example-usage/upload-from-web-browser.html`


Pass JSON with URLs so the server will download them and handle like they were uploaded:
```
-> POST /upload
   content-type: application/json
   data: [ { "url": "http://example.com/pic.jpg" }, { "url": "https://website.org/logo.png" } ]

<- 200 OK
   or 415 Unsupported Media Type if any downloaded file had unexpected MIME type
   or 400 Bad Request if invalid JSON was passed or the array was empty
   content-type: application/json
   body: ["WgyLB9IKorGE","RoydCP3nVXkI"] # list of images' IDs that were successfully uploaded
```
Example usage:
* `./example-usage/upload-json-url-with-curl.sh`


Pass JSON with Base64-encoded images (the array can actually mix `url`s and `base64`s):
```
-> POST /upload
   content-type: application/json
   data: [ { "base64": "(base64-encoded content of image)" } ]

<- 200 OK
   or 415 Unsupported Media Type if any encoded file had unexpected type
   or 400 Bad Request if invalid JSON was passed or the array was empty
   content-type: application/json
   body: ["WgyLB9IKorGE","RoydCP3nVXkI"] # list of images' IDs that were successfully uploaded
```
Example usage:
* `./example-usage/upload-json-base64-with-curl.sh`
Currently max request size with JSON body is 1 MiB.


On error, client only gets list of already uploaded images, or empty list if none succeed.


Run with `cargo run --release` or use `docker-compose build && docker-compose up` to build and run in standalone Docker container.
If your UID and/or GID is not 1000/1000, please change them in `.env` before running `docker-compose up`.
Otherwise, you may end up with root-owned `/tmp/uploads` and its contents. See your UID/GID by running `id` in shell.

When building on host, OpenCV libraries are required, v4 or v3.2 or v3.4.
When using v3.X, please fix opencv features in Cargo.toml to use "opencv-32" or "opencv-34" respectively.
