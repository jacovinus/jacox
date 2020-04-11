let express = require("express");
let PORT = process.env.PORT|| 666;
let app = express();

app.use(express.static("client"));

app.get('/',(req,res)=> {
  res.sendFile(__dirname + "/client/index.html")
})
  
const listener = app.listen(PORT,()=> {
  console.log("JACOX server is listening on port ",listener.address().port);
})