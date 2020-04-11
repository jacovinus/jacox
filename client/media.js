var videoArea = document.querySelector("video");
var videoSelect = document.querySelector("#camera")
async function getMedia(constraints) {
    let stream = null;
  
    try {
      stream = await navigator.mediaDevices.getUserMedia(constraints);
      /* use the stream */
      console.log(stream)
    } catch(err) {
      /* handle the error */
      console.log(err)
    }
  }


//MediaStreamTrack.getSources(getCameras);
function getCameras(sourceInfos) {
    for (var i = 0 ; i < sourceInfos.length; i++) {
        var sourceInfo = sourceInfos[i];
        var option = document.createElement('option');
        option.value = sourceInfo.id;
        if(sourceInfo.kind === 'video') {
            option.text = sourceInfo.label || 'camera' + (videoSelect.length + 1);
            videoSelect.appendChild(option);
        }
    }
}
startStream();
function startStream(){
navigator.getUserMedia = navigator.getUserMedia || navigator.webkitGetUserMedia || navigator.mozGetUserMedia;
var constraints = {audio : true, 
    video : {
        mandatory: {
            minWidth: 640,
            maxWidth: 640,
            minHeight: 360,
            maxHeight: 480
        }
    }};

    navigator.getUserMedia(constraints, onSuccess, onError);
}

function onSuccess(stream) {
    console.log("U r in the matrix biatch!")
    videoArea.srcObject = stream;
    videoArea.play();
}
function onError(error) {
    console.warn('error with getUSerMedia ',error);
}
