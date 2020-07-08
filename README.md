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
