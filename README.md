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
   or 400 Bad Request if invalid JSON was passed or the array of URLs was empty
```
Example usage:
* `./example-usage/upload-json-url-with-curl.sh`
