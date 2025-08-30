# voxa-server

Self host your voxa community server with customizable plugins

# Protocol

1. When the client is running in the background or windowed, it's connected to the [Voxa Cloud server](#voxa-cloud)

## Handshake

Client -> Cloud: `{ GenerateServerAuth: {} }`

Cloud -> Cloud: `{ ServerAuth: { auth: <Client-Temp-Auth> } }`

Client -> Server: Connect

Client -> Server: `{ auth: <Client-Temp-Auth> }`

Server -> Cloud: `GET /validate-auth?auth=<Client-Temp-Auth>`

Cloud -> Server: `{ 200 OK { username: <Username>, name: <Name>, id: <User-Id> } }`

## Sending a message

Client -> Server: `{ SendMessage: { content: <Message> } }`

Server -> Cloud: `POST /message { content: <Message>, author: <User-Id> }`

Cloud -> Client(s): `{ Message: { content: <Message>, author: <User-Id> } }`

# Voxa Cloud

The voxa cloud server is the main auth and notification handler.
