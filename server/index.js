let express = require('express');
let PORT = process.env.port || 666;
let app = express();
let server = require('http').Server(app);
let io = require('socket.io')(server);
app.use(express.static('client'))

server.listen(PORT,()=>{
    console.log("\x1b[36m",`Wellcome mortal \n the jacox server is up and running on por ${PORT}`)

    console.log('%c', 'padding:28px 119px;line-height:100px;background:red; no-repeat;')
})
io.on('connection',($socket)=>{
    console.log("el nodo con Ip " + socket.handshake.address + " Se ha conectado")
})