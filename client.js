import readline from 'readline';

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

const ws = new WebSocket('ws://localhost:7080');

ws.onopen = () => {
    console.log('WebSocket connection established');
};

ws.onmessage = (event) => {
    const message = JSON.parse(event.data);
    console.log('Received:', message);
    // ws.close();
};

function sendMessage(message) {
    ws.send(JSON.stringify({ type: 'send_message', params: {
        channel_id: 'general',
        contents: message
    }}));
}

function ask(question) {
  return new Promise((resolve) => {
    rl.question(question, resolve);
  });
}

while (true) {
    const message = await ask('');
    sendMessage(message);
}