import readline from 'readline';

async function createServerToken() {
    return (await (await fetch('http://localhost:3000/api/auth', {
        method: 'POST',
        body: JSON.stringify({
            intents: 'server',
            server_ip: '127.0.0.1'
        })
    })).json()).token;
}

const token = await createServerToken();

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

const ws = new WebSocket('ws://localhost:7080');

ws.onopen = () => {
    console.log('WebSocket connection established');
    console.log('Initializing handshake');
    ws.send(JSON.stringify({ version: '0.0.1', auth_token: token }))
};

ws.onmessage = (event) => {
    const message = JSON.parse(event.data);
    if (message.type === 'authenticated') {
        console.log('Logged in with uuid: ' + message.params.messages);
        console.log(message.params.messages);
    }
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
    if (message === 'q') {
        ws.close();
        process.exit();
    }
    sendMessage(message);
}