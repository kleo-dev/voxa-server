setInterval(() => {
    const ws = new WebSocket('ws://localhost:7080');

    function sendMessage(message) {
        ws.send(JSON.stringify({ type: 'send_message', params: {
            channel_id: 'Hello',
            contents: message
        }}));
    }

    ws.onopen = () => {
        console.log('WebSocket connection established');
        sendMessage('Hello, Server!');
    };

    ws.onmessage = (event) => {
        const message = JSON.parse(event.data);
        console.log('Received:', message);
        ws.close();
    };
}, 100)