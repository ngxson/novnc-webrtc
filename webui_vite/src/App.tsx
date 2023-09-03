import { useRef } from 'react';
import { VncScreen } from './lib';

function App() {
  const ref = useRef();

  return (
    <VncScreen
      url='http://localhost:8000/sdp'
      scaleViewport
      background="#000000"
      style={{
        width: '75vw',
        height: '75vh',
      }}
      ref={ref}
    />
  );
}

export default App
