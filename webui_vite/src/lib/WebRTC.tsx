import { default as _RFB } from '../noVNC/core/rfb';

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
  return new Promise((resolve, reject) => {
    let pc = new RTCPeerConnection({
      iceServers: iceServers ?? DEFAULT_ICE_SERVERS,
    });
    pc.oniceconnectionstatechange = e => console.log('WebRTC ICE state:', pc.iceConnectionState);
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

/**
 * Very thin wrapper for RFB
 */

/*
interface RFBWebRTC {
  capabilities: any;
  /////////////////////
  disconnect: () => any;
  sendCredentials: (credentials: any) => any;
  sendKey: (keysym: number, code: string, down?: boolean) => any;
  sendCtrlAltDel: () => any;
  focus: () => any;
  blur: () => any;
  machineShutdown: () => any;
  machineReboot: () => any;
  machineReset: () => any;
  clipboardPasteFrom: (text: string) => any;
  /////////////////////
  addEventListener: any;
  removeEventListener: any;
  /////////////////////
  viewOnly: boolean;
  focusOnClick: boolean;
  clipViewport: boolean;
  dragViewport: boolean;
  resizeSession: boolean;
  scaleViewport: boolean;
  showDotCursor: boolean;
  background: string;
  qualityLevel: number;
  compressionLevel: number;
}

class RFBWebRTC extends _RFB implements RFBWebRTC {
  constructor(target: any, dataChannel: string, options: any) {
    super(target, dataChannel, options);
  }
}

export default RFBWebRTC;
*/