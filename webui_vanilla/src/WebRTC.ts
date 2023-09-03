const DEFAULT_ICE_SERVERS = [{
  urls: 'stun:stun.l.google.com:19302'
}];

/**
 * Create a new RTCPeerConnection, then open a RTCDataChannel
 * @param sdpUrl 
 * @param iceServers 
 * @returns 
 */
export function getRTCDataChannel(sdpUrl: string, iceServers?: any): Promise<RTCDataChannel> {
  // @ts-ignore
  return new Promise((resolve, reject) => {
    let pc = new RTCPeerConnection({
      iceServers: iceServers ?? DEFAULT_ICE_SERVERS,
    });
    pc.oniceconnectionstatechange = () => console.log('WebRTC ICE state:', pc.iceConnectionState);
    pc.onicecandidate = async event => {
      if (event.candidate === null) {
        const localSD = pc.localDescription;
        console.log({ localSD });
        const rawResponse = await fetch(sdpUrl, {
          method: 'POST',
          headers: {
            'Accept': 'application/json',
            'Content-Type': 'application/json'
          },
          body: JSON.stringify(localSD),
        });
        const rawRemoteSD = await rawResponse.json();
        const remoteSD = new RTCSessionDescription(rawRemoteSD);
        console.log({ remoteSD });
        try {
          pc.setRemoteDescription(remoteSD);
        } catch (e) {
          console.error('setRemoteDescription', e);
        }
      }
    };
    pc.onnegotiationneeded = () => pc.createOffer()
      .then(d => pc.setLocalDescription(d))
      .catch(err => {
        console.error('setLocalDescription', err);
      });

    let dataChannel = pc.createDataChannel('vnc', {
      ordered: true
    });
    dataChannel.onopen = () => resolve(dataChannel);
  });
}
