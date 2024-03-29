#
# This is a Plumber API. You can run the API by clicking
# the 'Run API' button above.
#
# Find out more about building APIs with Plumber here:
#
#    https://www.rplumber.io/
#

library(plumber)

#* @apiTitle Plumber Example API
#* @apiDescription Plumber example description.

#* Echo back the input
#* @param msg The message to echo
#* @get /echo
function(msg = "") {
    list(msg = paste0("The message is: '", msg, "'"))
}

#* Return the sum of two numbers
#* @param a The first number to add
#* @param b The second number to add
#* @post /sum
function(a, b) {
    as.numeric(a) + as.numeric(b)
}

#* sleep for some time
#* @param zzz how long to sleep for 
#* @get /sleep
function(zzz) {
    Sys.sleep(zzz)
    paste0("I slept for ", zzz, " seconds")
}

#* Return the sum of two numbers
#* @param a The first number to add
#* @param b The second number to add
#* @post /sum
function(a, b) {
    as.numeric(a) + as.numeric(b)
}


#* @serializer pdf
#* @get /pdf
function(){
  plot(1:10, type="b")
  text(4, 8, "PDF from plumber!")
}

#* @serializer text
#* @get /text
function(){
  "just plain text here..."
}

#* @serializer html
#* @get /html
function(){
  "<html><h1>HTML!</h1>HTML here!</html>"
}

#* Download a binary file.
#* @serializer contentType list(type="application/octet-stream")
#* @get /download-binary
function(res){
  # TODO: Stream the data into the response rather than loading it all in memory
  # first.

  # Create a temporary example RDS file
  x <- list(a=123, b="hi!")
  tmp <- tempfile(fileext=".rds")
  saveRDS(x, tmp)

  # This header is a convention that instructs browsers to present the response
  # as a download named "mydata.Rds" rather than trying to render it inline.
  res$setHeader("Content-Disposition", "attachment; filename=mydata.Rds")

  # Read in the raw contents of the binary file
  bin <- readBin(tmp, "raw", n=file.info(tmp)$size)

  # Delete the temp file
  file.remove(tmp)

  # Return the binary contents
  bin
}